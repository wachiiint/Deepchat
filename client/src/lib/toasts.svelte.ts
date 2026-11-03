export type Toast = { id: number; text: string };

const LIFETIME_MS = 6000;
const MAX_VISIBLE = 3;

class ToastStore {
  items = $state<Toast[]>([]);
  #nextId = 1;
  #timers = new Map<number, ReturnType<typeof setTimeout>>();

  push(text: string) {
    // Repeating the same message restarts its timer instead of stacking a copy.
    const existing = this.items.find((t) => t.text === text);
    if (existing) {
      this.resume(existing.id);
      return;
    }
    const toast = { id: this.#nextId++, text };
    this.items = [...this.items, toast].slice(-MAX_VISIBLE);
    for (const id of this.#timers.keys()) {
      if (!this.items.some((t) => t.id === id)) this.pause(id);
    }
    this.resume(toast.id);
  }

  dismiss(id: number) {
    this.pause(id);
    this.items = this.items.filter((t) => t.id !== id);
  }

  pause(id: number) {
    clearTimeout(this.#timers.get(id));
    this.#timers.delete(id);
  }

  resume(id: number) {
    this.pause(id);
    this.#timers.set(
      id,
      setTimeout(() => this.dismiss(id), LIFETIME_MS),
    );
  }

  clearAll() {
    for (const id of [...this.#timers.keys()]) this.pause(id);
    this.items = [];
  }
}

export const toasts = new ToastStore();
