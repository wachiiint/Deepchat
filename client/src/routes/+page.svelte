<script lang="ts">
  import { onMount } from "svelte";
  import type { UnlistenFn } from "@tauri-apps/api/event";
  import * as chatApi from "$lib/chatApi";
  import type { ActiveRoom, Group, Message } from "$lib/types";
  import { avatarFill, initial } from "$lib/avatar";
  import { toasts } from "$lib/toasts.svelte";
  import ConnectView, {
    type ConnectPayload,
  } from "$lib/components/ConnectView.svelte";
  import UserList from "$lib/components/UserList.svelte";
  import GroupList from "$lib/components/GroupList.svelte";
  import ChatRoom from "$lib/components/ChatRoom.svelte";
  import GroupPreview from "$lib/components/GroupPreview.svelte";
  import Toasts from "$lib/components/Toasts.svelte";
  import Icon from "$lib/components/Icon.svelte";

  let connected = $state(false);
  let currentUser = $state("");
  let users = $state<string[]>([]);
  let groups = $state<Group[]>([]);
  let messages = $state<Message[]>([]);
  let activeRoom = $state<ActiveRoom | null>(null);
  let selectedPeer = $state<string | null>(null);
  let selectedGroupId = $state<string | null>(null);
  let previewGroupId = $state<string | null>(null);
  let busy = $state(false);

  let lastConnect = $state<ConnectPayload | null>(null);
  let connectError = $state("");
  let disconnectNotice = $state("");
  let createOpen = $state(false);
  let createError = $state("");
  let sendError = $state("");

  // Unread counts for rooms that are not open: groups by room id, private chats by peer name.
  let groupUnread = $state<Record<string, number>>({});
  let peerUnread = $state<Record<string, number>>({});

  let pendingGroup = $state<string | null>(null);
  let pendingAction = $state<"create" | "join" | null>(null);
  let pendingTimer: ReturnType<typeof setTimeout> | undefined;
  const PENDING_MS = 5000;

  // Which group to auto-open once the server's Groups push lists us in it.
  // Expires, so a no-op join (no reply, no push) can't auto-open a group later.
  function setPending(name: string, action: "create" | "join") {
    clearTimeout(pendingTimer);
    pendingGroup = name;
    pendingAction = action;
    pendingTimer = setTimeout(clearPending, PENDING_MS);
  }
  let usersRevision = 0;
  let groupsRevision = 0;
  let disconnectRevision = 0;
  let listenersReady: Promise<void>;
  let unlisteners: UnlistenFn[] = [];

  const activeGroup = $derived(
    activeRoom?.kind === "group"
      ? (groups.find((g) => g.id === activeRoom?.id) ?? null)
      : null,
  );
  const previewGroup = $derived(
    previewGroupId
      ? (groups.find((g) => g.id === previewGroupId) ?? null)
      : null,
  );
  const peerOnline = $derived(
    activeRoom?.kind !== "private" || users.includes(activeRoom.title),
  );

  function closeRoom() {
    activeRoom = null;
    selectedGroupId = null;
    selectedPeer = null;
    previewGroupId = null;
    messages = [];
    sendError = "";
  }

  function clearPending() {
    clearTimeout(pendingTimer);
    pendingGroup = null;
    pendingAction = null;
  }

  function resetSession() {
    connected = false;
    clearPending();
    closeRoom();
    users = [];
    groups = [];
    groupUnread = {};
    peerUnread = {};
    createOpen = false;
    createError = "";
    toasts.clearAll();
  }

  function report(e: unknown) {
    toasts.push(String(e));
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
    if (previewGroupId && !next.some((g) => g.id === previewGroupId)) {
      previewGroupId = null;
      selectedGroupId = null;
    }
    const joined = next.find(
      (g) => g.name === pendingGroup && g.members.includes(currentUser),
    );
    if (joined) {
      if (pendingAction === "create") {
        createOpen = false;
        createError = "";
      }
      clearPending();
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
          // A server error while a create is in flight belongs to the create box.
          // Otherwise keep the pending group: an unrelated error must not cancel the auto-open.
          if (pendingAction === "create") {
            createError = message;
            clearPending();
          } else toasts.push(message);
        }),
        chatApi.onDisconnected((message) => {
          disconnectRevision++;
          resetSession();
          connectError = "";
          disconnectNotice = message;
        }),
        chatApi.onMessageReceived((message) => {
          if (activeRoom && message.roomId === activeRoom.id) {
            if (!messages.some((m) => m.id === message.id)) {
              messages = [...messages, message];
            }
          } else if (message.sender !== currentUser) {
            if (message.roomId.startsWith("dm:")) {
              peerUnread[message.sender] =
                (peerUnread[message.sender] ?? 0) + 1;
            } else {
              groupUnread[message.roomId] =
                (groupUnread[message.roomId] ?? 0) + 1;
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

  async function handleConnect(payload: ConnectPayload) {
    busy = true;
    connectError = "";
    disconnectNotice = "";
    lastConnect = payload;
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
      connectError = String(e);
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
    clearPending();
    busy = true;
    try {
      const roomId = await chatApi.openPrivate(peer);
      closeRoom();
      selectedPeer = peer;
      activeRoom = { id: roomId, kind: "private", title: peer };
      delete peerUnread[peer];
      await loadMessages(roomId);
    } catch (e) {
      report(e);
    } finally {
      busy = false;
    }
  }

  async function selectGroup(group: Group) {
    if (!group.members.includes(currentUser)) {
      closeRoom();
      previewGroupId = group.id;
      selectedGroupId = group.id;
      return;
    }
    busy = true;
    try {
      closeRoom();
      selectedGroupId = group.id;
      activeRoom = { id: group.id, kind: "group", title: group.name };
      delete groupUnread[group.id];
      await loadMessages(group.id);
    } catch (e) {
      report(e);
    } finally {
      busy = false;
    }
  }

  async function handleCreateGroup(name: string) {
    busy = true;
    createError = "";
    try {
      setPending(name, "create");
      await chatApi.createGroup(name);
    } catch (e) {
      clearPending();
      createError = String(e);
    } finally {
      busy = false;
    }
  }

  function cancelCreate() {
    createOpen = false;
    createError = "";
  }

  async function handleJoinGroup(groupId: string) {
    busy = true;
    try {
      const name = groups.find((group) => group.id === groupId)?.name;
      if (name) setPending(name, "join");
      await chatApi.joinGroup(groupId);
    } catch (e) {
      clearPending();
      report(e);
    } finally {
      busy = false;
    }
  }

  async function handleSend(text: string) {
    if (!activeRoom) return false;
    busy = true;
    sendError = "";
    try {
      await chatApi.sendMessage(activeRoom.id, text);
      return true;
    } catch (e) {
      sendError = String(e);
      return false;
    } finally {
      busy = false;
    }
  }

  async function handleLeaveGroup(groupId: string) {
    clearPending();
    try {
      await chatApi.leaveGroup(groupId);
    } catch (e) {
      report(e);
    }
  }

  async function handleDisconnect() {
    try {
      await chatApi.disconnect();
      resetSession();
    } catch (e) {
      report(e);
    }
  }
</script>

{#if !connected}
  <ConnectView
    {busy}
    error={connectError}
    notice={disconnectNotice}
    initial={lastConnect}
    onconnect={handleConnect}
  />
{:else}
  <div class="shell">
    <aside class="sidebar">
      <div class="brand">Deepchat</div>
      <nav class="lists" aria-label="Rooms">
        <GroupList
          {groups}
          {currentUser}
          {selectedGroupId}
          unread={groupUnread}
          {busy}
          creating={pendingAction === "create"}
          {createError}
          bind:createOpen
          onselect={selectGroup}
          onjoin={handleJoinGroup}
          oncreate={handleCreateGroup}
          oncreatecancel={cancelCreate}
        />
        <UserList
          {users}
          {currentUser}
          {selectedPeer}
          unread={peerUnread}
          onselect={selectUser}
        />
      </nav>
      <div class="me">
        <span
          class="av"
          style:background={avatarFill(currentUser)}
          aria-hidden="true"
        >
          {initial(currentUser)}
        </span>
        <div class="who">
          <p class="name">Signed in as <strong>{currentUser}</strong></p>
          {#if lastConnect}
            <p class="server">
              <span class="dot" aria-hidden="true"
              ></span>{lastConnect.host}:{lastConnect.port}
            </p>
          {/if}
        </div>
        <button
          type="button"
          class="icon"
          aria-label="Disconnect"
          title="Disconnect"
          onclick={handleDisconnect}
        >
          <Icon name="logout" />
        </button>
      </div>
    </aside>
    <main class="main">
      {#if previewGroup}
        <GroupPreview
          group={previewGroup}
          {busy}
          joining={pendingAction === "join" &&
            pendingGroup === previewGroup.name}
          onjoin={handleJoinGroup}
        />
      {:else}
        <ChatRoom
          room={activeRoom}
          group={activeGroup}
          {messages}
          {currentUser}
          {busy}
          {peerOnline}
          {sendError}
          onsend={handleSend}
          onleave={handleLeaveGroup}
          oneditdraft={() => (sendError = "")}
        />
      {/if}
    </main>
  </div>
  <Toasts />
{/if}

<style>
  .shell {
    display: grid;
    grid-template-columns: 264px 1fr;
    height: 100vh;
    min-height: 0;
    background: var(--bg);
  }

  .sidebar {
    display: grid;
    grid-template-rows: 56px 1fr auto;
    min-height: 0;
    background: var(--side);
    border-right: 1px solid var(--line);
  }

  .brand {
    display: flex;
    align-items: center;
    padding: 0 16px;
    border-bottom: 1px solid var(--line);
    font-size: 16px;
    font-weight: 650;
    letter-spacing: -0.01em;
  }

  .lists {
    overflow-y: auto;
    min-height: 0;
  }

  .me {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px 12px 12px 16px;
    border-top: 1px solid var(--line);
    min-width: 0;
  }

  .av {
    width: 32px;
    height: 32px;
    border-radius: var(--radius-md);
    display: grid;
    place-items: center;
    flex: none;
    color: #fff;
    font-weight: 700;
    font-size: 13px;
  }

  .who {
    flex: 1;
    min-width: 0;
    line-height: 1.3;
  }

  .name,
  .server {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .name {
    font-size: 12px;
    color: var(--mute);
  }

  .name strong {
    display: block;
    font-size: 14px;
    color: var(--ink);
    font-weight: 600;
  }

  .server {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--faint);
    font-variant-numeric: tabular-nums;
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--ok);
    flex: none;
  }

  .icon {
    width: 32px;
    height: 32px;
    display: grid;
    place-items: center;
    border: 0;
    border-radius: var(--radius-sm);
    background: none;
    color: var(--mute);
    flex: none;
  }

  .icon:hover {
    background: var(--raise);
    color: var(--ink);
  }

  .main {
    min-width: 0;
    min-height: 0;
  }
</style>
