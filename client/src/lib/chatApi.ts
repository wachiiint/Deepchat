import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ConnectResult, Group, Message } from "./types";

export async function connect(
  host: string,
  port: number,
  name: string,
): Promise<ConnectResult> {
  return invoke("connect", { host, port, name });
}

export async function listUsers(): Promise<string[]> {
  return invoke("list_users");
}

export async function listGroups(): Promise<Group[]> {
  return invoke("list_groups");
}

export async function createGroup(name: string): Promise<void> {
  return invoke("create_group", { name });
}

export async function joinGroup(groupId: string): Promise<void> {
  return invoke("join_group", { groupId });
}

export async function openPrivate(peer: string): Promise<string> {
  return invoke("open_private", { peer });
}

export async function sendMessage(roomId: string, text: string): Promise<void> {
  return invoke("send_message", { roomId, text });
}

export async function getMessages(roomId: string): Promise<Message[]> {
  return invoke("get_messages", { roomId });
}

export async function leaveGroup(groupId: string): Promise<void> {
  return invoke("leave_group", { groupId });
}

export async function deleteGroup(groupId: string): Promise<void> {
  return invoke("delete_group", { groupId });
}

export async function disconnect(): Promise<void> {
  return invoke("disconnect");
}

export function onChatError(
  handler: (message: string) => void,
): Promise<UnlistenFn> {
  return listen<string>("chat-error", (event) => handler(event.payload));
}

export function onDisconnected(
  handler: (message: string) => void,
): Promise<UnlistenFn> {
  return listen<string>("disconnected", (event) => handler(event.payload));
}

export function onUsersUpdated(
  handler: (users: string[]) => void,
): Promise<UnlistenFn> {
  return listen<string[]>("users-updated", (event) => handler(event.payload));
}

export function onGroupsUpdated(
  handler: (groups: Group[]) => void,
): Promise<UnlistenFn> {
  return listen<Group[]>("groups-updated", (event) => handler(event.payload));
}

export function onMessageReceived(
  handler: (message: Message) => void,
): Promise<UnlistenFn> {
  return listen<Message>("message-received", (event) => handler(event.payload));
}

export function onGroupDeleted(
  handler: (groupId: string) => void,
): Promise<UnlistenFn> {
  return listen<string>("group-deleted", (event) => handler(event.payload));
}
