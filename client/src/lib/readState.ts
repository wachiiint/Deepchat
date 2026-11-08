import type { Message, ReadReceipt } from "./types";

/** In-memory only, like chat history. All counters reset on a new connection. */
export class ReadState {
  private unread = new Map<string, number>();
  private latest = new Map<string, string>();
  private sent = new Map<string, string>();

  counts(): Record<string, number> {
    return Object.fromEntries(this.unread);
  }

  receive(
    message: Message,
    me: string,
    activeRoom: string | null,
    focused: boolean,
  ) {
    const previous = this.latest.get(message.roomId) ?? "0";
    if (BigInt(message.id) <= BigInt(previous)) return;
    this.latest.set(message.roomId, message.id);
    if (message.roomId === activeRoom && focused)
      this.unread.delete(message.roomId);
    else if (message.sender !== me) {
      this.unread.set(
        message.roomId,
        (this.unread.get(message.roomId) ?? 0) + 1,
      );
    }
  }

  observeHistory(roomId: string, messages: Message[]) {
    for (const message of messages) {
      if (
        message.roomId === roomId &&
        BigInt(message.id) > BigInt(this.latest.get(roomId) ?? "0")
      ) {
        this.latest.set(roomId, message.id);
      }
    }
  }

  visible(roomId: string, focused: boolean): string | null {
    if (!focused) return null;
    this.unread.delete(roomId);
    const latest = this.latest.get(roomId) ?? "0";
    return BigInt(latest) > BigInt(this.sent.get(roomId) ?? "0")
      ? latest
      : null;
  }

  acknowledge(roomId: string, upto: string) {
    if (BigInt(upto) > BigInt(this.sent.get(roomId) ?? "0"))
      this.sent.set(roomId, upto);
  }

  retainGroups(groups: Set<string>) {
    for (const roomId of this.latest.keys()) {
      if (roomId.startsWith("group:") && !groups.has(roomId)) this.drop(roomId);
    }
  }

  drop(roomId: string) {
    this.unread.delete(roomId);
    this.latest.delete(roomId);
    this.sent.delete(roomId);
  }

  reset() {
    this.unread.clear();
    this.latest.clear();
    this.sent.clear();
  }
}

export function applyReceipt(
  messages: Message[],
  receipt: ReadReceipt,
  me: string,
): Message[] {
  if (receipt.by === me) return messages;
  return messages.map((message) =>
    message.roomId === receipt.roomId &&
    message.sender === me &&
    BigInt(message.id) <= BigInt(receipt.upto)
      ? {
          ...message,
          readers: [...new Set([...message.readers, receipt.by])].sort(),
        }
      : message,
  );
}

export function mergeMessages(
  history: Message[],
  current: Message[],
  receipts: ReadReceipt[] = [],
  me = "",
): Message[] {
  const byId = new Map(history.map((message) => [message.id, message]));
  for (const message of current) {
    const prior = byId.get(message.id);
    byId.set(message.id, {
      ...message,
      readers: [
        ...new Set([...(prior?.readers ?? []), ...message.readers]),
      ].sort(),
    });
  }
  let merged = [...byId.values()].sort((a, b) =>
    BigInt(a.id) < BigInt(b.id) ? -1 : BigInt(a.id) > BigInt(b.id) ? 1 : 0,
  );
  for (const receipt of receipts) merged = applyReceipt(merged, receipt, me);
  return merged;
}

export function newestSeen(
  messages: Message[],
  me: string,
): Message | undefined {
  return messages.findLast(
    (message) => message.sender === me && message.readers.length > 0,
  );
}
