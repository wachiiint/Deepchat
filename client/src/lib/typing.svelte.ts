// Who is typing, per room. The server sends no "stopped typing", so every signal
// is a lease: it lapses TTL_MS after the last one. Lives only in the frontend.
export const TYPING_TTL_MS = 4000;

class TypingStore {
  rooms = $state<Record<string, string[]>>({});
  #timers = new Map<string, ReturnType<typeof setTimeout>>();

  /** Others typing in `roomId`, in the order they started. */
  in(roomId: string): string[] {
    return this.rooms[roomId] ?? [];
  }

  /** A signal arrived: add `name` if new and restart their lease. */
  note(roomId: string, name: string) {
    if (!this.in(roomId).includes(name)) {
      this.rooms[roomId] = [...this.in(roomId), name];
    }
    const key = this.#key(roomId, name);
    clearTimeout(this.#timers.get(key));
    this.#timers.set(
      key,
      setTimeout(() => this.clear(roomId, name), TYPING_TTL_MS),
    );
  }

  /** Drop one person, e.g. because their message arrived. */
  clear(roomId: string, name: string) {
    const key = this.#key(roomId, name);
    clearTimeout(this.#timers.get(key));
    this.#timers.delete(key);
    const rest = this.in(roomId).filter((n) => n !== name);
    if (rest.length) this.rooms[roomId] = rest;
    else delete this.rooms[roomId];
  }

  /** The room is gone (deleted group, left group). */
  clearRoom(roomId: string) {
    for (const name of this.in(roomId)) this.clear(roomId, name);
  }

  /** Drop everyone who is not in `online`: a disconnected user cannot still be typing. */
  prune(online: string[]) {
    for (const [roomId, names] of Object.entries(this.rooms)) {
      for (const name of names) {
        if (!online.includes(name)) this.clear(roomId, name);
      }
    }
  }

  clearAll() {
    for (const timer of this.#timers.values()) clearTimeout(timer);
    this.#timers.clear();
    this.rooms = {};
  }

  // NUL cannot appear in a display name typed into the connect form.
  #key(roomId: string, name: string) {
    return `${roomId}\0${name}`;
  }
}

export const typing = new TypingStore();
