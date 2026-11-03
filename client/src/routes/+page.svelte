<script lang="ts">
  import { onMount } from "svelte";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import * as chatApi from "$lib/chatApi";
  import type { ActiveRoom, Group, Message } from "$lib/types";
  import ConnectView from "$lib/components/ConnectView.svelte";
  import UserList from "$lib/components/UserList.svelte";
  import GroupList from "$lib/components/GroupList.svelte";
  import ChatRoom from "$lib/components/ChatRoom.svelte";

  let connected = $state(false);
  let currentUser = $state("");
  let users = $state<string[]>([]);
  let groups = $state<Group[]>([]);
  let messages = $state<Message[]>([]);
  let activeRoom = $state<ActiveRoom | null>(null);
  let selectedPeer = $state<string | null>(null);
  let selectedGroupId = $state<string | null>(null);
  let busy = $state(false);
  let error = $state("");

  let pendingGroup = $state<string | null>(null);
  let pendingTimer: ReturnType<typeof setTimeout> | undefined;
  const PENDING_MS = 5000;

  // Which group to auto-open once the server's Groups push lists us in it.
  // Expires, so a no-op join (no reply, no push) can't auto-open a group later.
  function setPending(name: string | null) {
    clearTimeout(pendingTimer);
    pendingGroup = name;
    if (name !== null) {
      pendingTimer = setTimeout(() => (pendingGroup = null), PENDING_MS);
    }
  }

  let usersRevision = 0;
  let groupsRevision = 0;
  let disconnectRevision = 0;
  let listenersReady: Promise<void>;
  let unlisteners: UnlistenFn[] = [];

  function closeRoom() {
    activeRoom = null;
    selectedGroupId = null;
    selectedPeer = null;
    messages = [];
  }

  function updateGroups(next: Group[]) {
    groupsRevision++;
    groups = next;
    if (
      activeRoom?.kind === "group" &&
      !next.some(
        (g) => g.id === activeRoom?.id && g.members.includes(currentUser),
      )
    ) {
      closeRoom();
    }
    const joined = next.find(
      (g) => g.name === pendingGroup && g.members.includes(currentUser),
    );
    if (joined) {
      setPending(null);
      void selectGroup(joined);
    }
  }

  onMount(() => {
    let cancelled = false;

    listenersReady = (async () => {
      const offs = await Promise.all([
        chatApi.onUsersUpdated((next) => {
          usersRevision++;

          users = next;
        }),
        chatApi.onGroupsUpdated((next) => {
          updateGroups(next);
        }),
        chatApi.onChatError((message) => {
          error = message;
        }),
        chatApi.onDisconnected((message) => {
          disconnectRevision++;
          connected = false;
          setPending(null);
          closeRoom();
          users = [];
          groups = [];
          error = message;
        }),
        chatApi.onMessageReceived((message) => {
          if (activeRoom && message.roomId === activeRoom.id) {
            if (!messages.some((m) => m.id === message.id)) {
              messages = [...messages, message];
            }
          }
        }),
      ]);
      if (cancelled) {
        offs.forEach((off) => off());
        return;
      }
      unlisteners = offs;
    })();

    return () => {
      cancelled = true;
      unlisteners.forEach((off) => off());
    };
  });

  async function handleConnect(payload: {
    host: string;
    port: number;
    name: string;
  }) {
    busy = true;
    error = "";
    try {
      await listenersReady;
      currentUser = payload.name;
      const initialDisconnectRevision = disconnectRevision;
      const initialUsersRevision = usersRevision;
      const initialGroupsRevision = groupsRevision;
      const result = await chatApi.connect(
        payload.host,
        payload.port,
        payload.name,
      );
      if (disconnectRevision !== initialDisconnectRevision) return;
      currentUser = result.name;
      if (usersRevision === initialUsersRevision) users = result.users;
      if (groupsRevision === initialGroupsRevision) groups = result.groups;
      connected = true;
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function loadMessages(roomId: string) {
    const history = await chatApi.getMessages(roomId);
    if (activeRoom?.id === roomId) {
      const byId = new Map(history.map((message) => [message.id, message]));
      for (const message of messages)
        if (message.roomId === roomId) byId.set(message.id, message);
      messages = [...byId.values()];
    }
  }

  async function selectUser(peer: string) {
    if (peer === currentUser) return;
    setPending(null);
    busy = true;
    error = "";
    try {
      const roomId = await chatApi.openPrivate(peer);
      selectedPeer = peer;
      selectedGroupId = null;
      activeRoom = { id: roomId, kind: "private", title: peer };
      messages = [];
      await loadMessages(roomId);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function selectGroup(group: Group) {
    if (!group.members.includes(currentUser)) {
      error = "Join this group before opening the chat room.";
      return;
    }
    busy = true;
    error = "";
    try {
      selectedGroupId = group.id;
      selectedPeer = null;
      activeRoom = { id: group.id, kind: "group", title: group.name };
      messages = [];
      await loadMessages(group.id);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function handleCreateGroup(name: string) {
    busy = true;
    error = "";
    try {
      setPending(name);
      await chatApi.createGroup(name);
    } catch (e) {
      setPending(null);
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function handleJoinGroup(groupId: string) {
    busy = true;
    error = "";
    try {
      setPending(groups.find((group) => group.id === groupId)?.name ?? null);
      await chatApi.joinGroup(groupId);
    } catch (e) {
      setPending(null);
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function handleSend(text: string) {
    if (!activeRoom) return false;
    busy = true;
    error = "";
    try {
      await chatApi.sendMessage(activeRoom.id, text);
      return true;
    } catch (e) {
      error = String(e);
      return false;
    } finally {
      busy = false;
    }
  }

  async function handleLeaveGroup(groupId: string) {
    setPending(null);
    error = "";
    try {
      await chatApi.leaveGroup(groupId);
    } catch (e) {
      error = String(e);
    }
  }

  async function handleDisconnect() {
    try {
      await chatApi.disconnect();
      connected = false;
      closeRoom();
      groups = [];
      users = [];
      setPending(null);
    } catch (e) {
      error = String(e);
    }
  }
</script>

{#if !connected}
  <ConnectView {busy} {error} onconnect={handleConnect} />
{:else}
  <div class="shell">
    <aside class="sidebar">
      <div class="sidebar-top">
        <p class="brand">Deepchat</p>
        <p class="you">Signed in as <strong>{currentUser}</strong></p>
        <button type="button" onclick={handleDisconnect}>Disconnect</button>
      </div>
      <div class="lists">
        <UserList {users} {currentUser} {selectedPeer} onselect={selectUser} />
        <GroupList
          {groups}
          {currentUser}
          {selectedGroupId}
          onselect={selectGroup}
          onjoin={handleJoinGroup}
          oncreate={handleCreateGroup}
          onleave={handleLeaveGroup}
          {busy}
        />
      </div>
      {#if error}
        <p class="sidebar-error" role="alert">{error}</p>
      {/if}
    </aside>
    <main class="main">
      <ChatRoom
        room={activeRoom}
        {messages}
        {currentUser}
        {busy}
        onsend={handleSend}
      />
    </main>
  </div>
{/if}

<style>
  .shell {
    display: grid;
    grid-template-columns: minmax(240px, 300px) 1fr;
    height: 100vh;
    min-height: 0;
    background: linear-gradient(180deg, #efeae1 0%, #f3f0ea 40%), #f3f0ea;
  }

  .sidebar {
    display: grid;
    grid-template-rows: auto 1fr auto;
    min-height: 0;
    border-right: 1px solid #ddd6ca;
    background: rgba(255, 252, 247, 0.88);
  }

  .sidebar-top {
    padding: 1rem 1rem 0.75rem;
    border-bottom: 1px solid #ddd6ca;
  }

  .brand {
    margin: 0;
    font-family: "Fraunces", "Iowan Old Style", Georgia, serif;
    font-size: 1.5rem;
    color: #1f3d34;
  }

  .you {
    margin: 0.25rem 0 0;
    font-size: 0.9rem;
    color: #6a655c;
  }

  .lists {
    display: grid;
    grid-template-rows: 1fr 1.2fr;
    min-height: 0;
    overflow: hidden;
  }

  .sidebar-error {
    margin: 0;
    padding: 0.65rem 1rem;
    border-top: 1px solid #f0c7c7;
    background: #fff5f5;
    color: #9b2c2c;
    font-size: 0.85rem;
  }

  .main {
    min-width: 0;
    min-height: 0;
  }

  @media (max-width: 720px) {
    .shell {
      grid-template-columns: 1fr;
      grid-template-rows: minmax(220px, 40vh) 1fr;
    }

    .sidebar {
      border-right: none;
      border-bottom: 1px solid #ddd6ca;
    }
  }
</style>
