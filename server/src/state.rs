//! In-memory server state. Owned by the hub task only, so no locks.

use std::collections::hash_map::Entry;
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

use tokio::sync::mpsc;

use protocol::{ErrorKind, GroupInfo, ServerMsg};

pub type ConnId = u64;
pub type Outbox = mpsc::UnboundedSender<ServerMsg>;

struct Conn {
    tx: Outbox,
    name: Option<String>, // None until Register succeeds
}

pub const MAX_BOTS: usize = 3;
pub const MAX_PERSONALITY_DESC: usize = 500;
const HISTORY: usize = 50; // group lines a bot sees

struct Group {
    creator: String,
    members: BTreeSet<String>,
    bots: BTreeMap<String, String>, // bot name -> personality
    history: VecDeque<String>,      // "from: text", newest last
}

/// A bot reply to fetch from the LLM. The hub queues it; `hub::run` spawns the call.
#[derive(Debug)]
pub struct AiJob {
    pub group: String,
    pub bot: String,
    pub system: String,
    pub prompt: String,
}

#[derive(Default)]
pub struct State {
    conns: HashMap<ConnId, Conn>,
    names: HashMap<String, ConnId>,
    groups: HashMap<String, Group>,
    // TODO(ai):
    // - Drop this pub queue (State must not feed the hub, which owns it): make
    //   `ask_bot(&self) -> Option<AiJob>` and have `hub::apply` return the job to `run`.
    // - `ask_bot`: join history via `iter()`, not `Vec::from(history.clone())`.
    // - `spawn_ai`: cap concurrent calls (per bot, or a global Semaphore).
    // - `bot_reply`: skip if the bot/group is gone; don't `record` "(bot unavailable)" errors.
    pub ai_jobs: Vec<AiJob>,
}

impl State {
    pub fn connect(&mut self, id: ConnId, tx: Outbox) {
        self.conns.insert(id, Conn { tx, name: None });
    }

    /// Returns the name if the connection had registered.
    pub fn disconnect(&mut self, id: ConnId) -> Option<String> {
        let name = self.conns.remove(&id)?.name?;
        self.names.remove(&name);
        Some(name)
    }

    pub fn register(&mut self, id: ConnId, name: String) -> Result<(), ErrorKind> {
        if self.names.contains_key(&name) {
            return Err(ErrorKind::NameTaken);
        }
        let Some(conn) = self.conns.get_mut(&id) else {
            return Ok(()); // already disconnected
        };
        conn.name = Some(name.clone());
        self.names.insert(name, id);
        Ok(())
    }

    pub fn name_of(&self, id: ConnId) -> Option<&str> {
        self.conns.get(&id)?.name.as_deref()
    }

    pub fn id_of(&self, name: &str) -> Option<ConnId> {
        self.names.get(name).copied()
    }

    /// Registered names, sorted so every client shows the same order.
    pub fn names(&self) -> Vec<String> {
        let mut v: Vec<String> = self.names.keys().cloned().collect();
        v.sort();
        v
    }

    pub fn create_group(&mut self, group: String, creator: &str) -> Result<(), ErrorKind> {
        match self.groups.entry(group) {
            Entry::Occupied(_) => Err(ErrorKind::GroupExists),
            Entry::Vacant(e) => {
                e.insert(Group {
                    creator: creator.to_owned(),
                    members: BTreeSet::from([creator.to_owned()]),
                    bots: BTreeMap::new(),
                    history: VecDeque::new(),
                });
                Ok(())
            }
        }
    }

    /// `Ok(true)` if `name` just joined, `Ok(false)` if it already was a member.
    pub fn join_group(&mut self, group: &str, name: String) -> Result<bool, ErrorKind> {
        let group = self.groups.get_mut(group).ok_or(ErrorKind::UnknownGroup)?;
        Ok(group.members.insert(name))
    }

    /// Deletes the group when its last member leaves.
    pub fn leave_group(&mut self, group: &str, name: &str) -> Result<(), ErrorKind> {
        let info = self.groups.get_mut(group).ok_or(ErrorKind::UnknownGroup)?;
        if !info.members.remove(name) {
            return Err(ErrorKind::NotMember);
        }
        if info.members.is_empty() {
            self.groups.remove(group);
        }
        Ok(())
    }

    /// Creator-only. Removes the group for everyone, even if the creator already left it.
    /// Returns who to notify: the members at the time of deletion, plus the creator.
    pub fn delete_group(&mut self, group: &str, name: &str) -> Result<BTreeSet<String>, ErrorKind> {
        let info = self.groups.get(group).ok_or(ErrorKind::UnknownGroup)?;
        if info.creator != name {
            return Err(ErrorKind::NotCreator);
        }
        let mut info = self.groups.remove(group).ok_or(ErrorKind::UnknownGroup)?;
        info.members.insert(info.creator);
        Ok(info.members)
    }

    /// Creator-only. A bot name must be unique in its group and must not be a user's name.
    pub fn create_bot(
        &mut self,
        group: &str,
        by: &str,
        name: String,
        personality: String,
    ) -> Result<(), ErrorKind> {
        let info = self.groups.get_mut(group).ok_or(ErrorKind::UnknownGroup)?;
        if info.creator != by {
            return Err(ErrorKind::NotCreator);
        }
        if info.bots.contains_key(&name) || self.names.contains_key(&name) {
            return Err(ErrorKind::BotNameTaken);
        }
        if info.bots.len() >= MAX_BOTS || personality.chars().count() > MAX_PERSONALITY_DESC {
            return Err(ErrorKind::BotLimit);
        }
        info.bots.insert(name, personality);
        Ok(())
    }

    /// Creator-only. `Ok(false)` if the group had no such bot.
    pub fn remove_bot(&mut self, group: &str, by: &str, name: &str) -> Result<bool, ErrorKind> {
        let info = self.groups.get_mut(group).ok_or(ErrorKind::UnknownGroup)?;
        if info.creator != by {
            return Err(ErrorKind::NotCreator);
        }
        Ok(info.bots.remove(name).is_some())
    }

    /// Remembers a group line so bots get context. Keeps only the last `HISTORY`.
    pub fn record(&mut self, group: &str, from: &str, text: &str) {
        if let Some(info) = self.groups.get_mut(group) {
            info.history.push_back(format!("{from}: {text}"));
            if info.history.len() > HISTORY {
                info.history.pop_front();
            }
        }
    }

    /// If `text` starts with `@<bot>` of this group, queue an AI job and return the bot.
    pub fn ask_bot(&mut self, group: &str, text: &str) -> Option<String> {
        let info = self.groups.get(group)?;
        let mention = text.strip_prefix('@')?.split_whitespace().next()?;
        let (bot, personality) = info.bots.get_key_value(mention)?;
        self.ai_jobs.push(AiJob {
            group: group.to_owned(),
            bot: bot.clone(),
            system: format!("You are {bot}, a bot in a group chat. {personality}"),
            // Persona repeated after the log: small models otherwise just continue the log.
            prompt: format!(
                "Group chat so far:\n{}\n\nYou are {bot}. {personality}\n\
                 Write only {bot}'s short reply to the last message. Do not start it with @.",
                Vec::from(info.history.clone()).join("\n")
            ),
        });
        Some(bot.clone())
    }

    /// Members of `group`, with no membership check (for bot replies).
    pub fn members(&self, group: &str) -> Option<&BTreeSet<String>> {
        self.groups.get(group).map(|g| &g.members)
    }

    /// Members of `group`, but only if `name` is one of them.
    pub fn members_for(&self, group: &str, name: &str) -> Result<&BTreeSet<String>, ErrorKind> {
        let group = self.groups.get(group).ok_or(ErrorKind::UnknownGroup)?;
        if group.members.contains(name) {
            Ok(&group.members)
        } else {
            Err(ErrorKind::NotMember)
        }
    }

    pub fn group_list(&self) -> Vec<GroupInfo> {
        let mut groups: Vec<_> = self
            .groups
            .iter()
            .map(|(name, group)| GroupInfo {
                group: name.clone(),
                creator: group.creator.clone(),
                members: group.members.iter().cloned().collect(),
                bots: group.bots.keys().cloned().collect(),
            })
            .collect();
        groups.sort_by(|a, b| a.group.cmp(&b.group));
        groups
    }

    /// Removes `name` from every group and deletes groups left empty.
    /// Returns `true` if `name` was in at least one group, i.e. the group list changed
    /// and clients need a fresh `Groups` push.
    pub fn remove_member(&mut self, name: &str) -> bool {
        let mut changed = false;
        self.groups.retain(|_, group| {
            changed |= group.members.remove(name);
            !group.members.is_empty()
        });
        changed
    }

    pub fn send(&self, id: ConnId, msg: ServerMsg) {
        if let Some(conn) = self.conns.get(&id) {
            // Err = writer task already gone; Disconnect event will clean up
            let _ = conn.tx.send(msg);
        }
    }

    pub fn broadcast(&self, msg: &ServerMsg) {
        for &id in self.names.values() {
            self.send(id, msg.clone());
        }
    }
}

#[cfg(test)]
mod bot_tests {
    use super::*;

    /// Group "g" created by "a".
    fn state() -> State {
        let mut s = State::default();
        s.create_group("g".into(), "a").unwrap();
        s
    }

    fn add(s: &mut State, by: &str, name: &str) -> Result<(), ErrorKind> {
        s.create_bot("g", by, name.into(), "You talk like a pirate.".into())
    }

    #[test]
    fn creator_adds_and_removes_bot() {
        let mut s = state();
        assert_eq!(add(&mut s, "a", "Pirate"), Ok(()));
        assert_eq!(s.remove_bot("g", "a", "Pirate"), Ok(true));
        assert_eq!(s.remove_bot("g", "a", "Pirate"), Ok(false));
    }

    #[test]
    fn non_creator_rejected() {
        let mut s = state();
        s.join_group("g", "b".into()).unwrap();
        assert_eq!(add(&mut s, "b", "Pirate"), Err(ErrorKind::NotCreator));
        add(&mut s, "a", "Pirate").unwrap();
        assert_eq!(s.remove_bot("g", "b", "Pirate"), Err(ErrorKind::NotCreator));
    }

    #[test]
    fn at_most_three_bots() {
        let mut s = state();
        for name in ["B1", "B2", "B3"] {
            add(&mut s, "a", name).unwrap();
        }
        assert_eq!(add(&mut s, "a", "B4"), Err(ErrorKind::BotLimit));
    }

    #[test]
    fn personality_length_limit() {
        let mut s = state();
        let ok = "x".repeat(MAX_PERSONALITY_DESC);
        let long = "x".repeat(MAX_PERSONALITY_DESC + 1);
        assert_eq!(s.create_bot("g", "a", "Ok".into(), ok), Ok(()));
        assert_eq!(
            s.create_bot("g", "a", "Long".into(), long),
            Err(ErrorKind::BotLimit)
        );
    }

    #[test]
    fn bot_name_clashes_rejected() {
        let mut s = state();
        s.connect(1, mpsc::unbounded_channel().0);
        s.register(1, "bob".into()).unwrap();
        add(&mut s, "a", "Pirate").unwrap();
        assert_eq!(add(&mut s, "a", "Pirate"), Err(ErrorKind::BotNameTaken));
        assert_eq!(add(&mut s, "a", "bob"), Err(ErrorKind::BotNameTaken));
    }

    #[test]
    fn bots_vanish_with_group() {
        let mut s = state();
        add(&mut s, "a", "Pirate").unwrap();
        s.delete_group("g", "a").unwrap();
        s.create_group("g".into(), "a").unwrap();
        assert_eq!(add(&mut s, "a", "Pirate"), Ok(())); // no clash: old bot is gone
    }
}
