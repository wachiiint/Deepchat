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

  let unlisteners: UnlistenFn[] = [];

  onMount(() => {
    let cancelled = false;

    (async () => {
      const offs = await Promise.all([
        chatApi.onUsersUpdated((next) => {
          users = next;
        }),
        chatApi.onGroupsUpdated((next) => {
          groups = next;
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
      const result = await chatApi.connect(
        payload.host,
        payload.port,
        payload.name,
      );
      currentUser = result.name;
      users = result.users;
      groups = result.groups;
      connected = true;
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function loadMessages(roomId: string) {
    messages = await chatApi.getMessages(roomId);
  }

  async function selectUser(peer: string) {
    if (peer === currentUser) return;
    busy = true;
    error = "";
    try {
      const roomId = await chatApi.openPrivate(peer);
      selectedPeer = peer;
      selectedGroupId = null;
      activeRoom = { id: roomId, kind: "private", title: peer };
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
      const group = await chatApi.createGroup(name);
      groups = await chatApi.listGroups();
      await selectGroup(group);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function handleJoinGroup(groupId: string) {
    busy = true;
    error = "";
    try {
      const group = await chatApi.joinGroup(groupId);
      groups = await chatApi.listGroups();
      await selectGroup(group);
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function handleSend(text: string) {
    if (!activeRoom) return;
    busy = true;
    error = "";
    try {
      const message = await chatApi.sendMessage(activeRoom.id, text);
      if (!messages.some((m) => m.id === message.id)) {
        messages = [...messages, message];
      }
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function handleSimulate() {
    if (!activeRoom) return;
    busy = true;
    error = "";
    try {
      const message = await chatApi.simulateIncoming(activeRoom.id);
      if (!messages.some((m) => m.id === message.id)) {
        messages = [...messages, message];
      }
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
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
      </div>
      <div class="lists">
        <UserList
          {users}
          {currentUser}
          {selectedPeer}
          onselect={selectUser}
        />
        <GroupList
          {groups}
          {currentUser}
          {selectedGroupId}
          onselect={selectGroup}
          onjoin={handleJoinGroup}
          oncreate={handleCreateGroup}
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
        onsimulate={handleSimulate}
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
    background:
      linear-gradient(180deg, #efeae1 0%, #f3f0ea 40%),
      #f3f0ea;
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
