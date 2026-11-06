import test from "node:test";
import assert from "node:assert/strict";
import {
  ReadState,
  applyReceipt,
  mergeMessages,
  newestSeen,
} from "./readState.ts";
import type { Message } from "./types";

function message(id: string, sender = "a", roomId = "dm:a"): Message {
  return {
    id,
    roomId,
    sender,
    text: "hello",
    timestamp: 0,
    peer: roomId.startsWith("dm:") ? sender : null,
    readers: [],
  };
}

test("closed rooms count incoming messages, but never own echoes or duplicates", () => {
  const state = new ReadState();
  state.receive(message("1"), "b", null, true);
  state.receive(message("2"), "b", "other", true);
  state.receive(message("2"), "b", "other", true);
  state.receive(message("3", "b"), "b", "other", true);
  assert.deepEqual(state.counts(), { "dm:a": 2 });
});

test("a focused open room clears unread and an unfocused room does not send receipts", () => {
  const state = new ReadState();
  state.receive(message("1"), "b", "dm:a", true);
  assert.deepEqual(state.counts(), {});
  state.receive(message("2"), "b", "dm:a", false);
  assert.deepEqual(state.counts(), { "dm:a": 1 });
  assert.equal(state.visible("dm:a", false), null);
  assert.deepEqual(state.counts(), { "dm:a": 1 });
  assert.equal(state.visible("dm:a", true), "2");
  assert.deepEqual(state.counts(), {});
  // Failed sends can retry; successful sends do not repeat the same watermark.
  assert.equal(state.visible("dm:a", true), "2");
  state.acknowledge("dm:a", "2");
  assert.equal(state.visible("dm:a", true), null);
  state.receive(message("4"), "b", "dm:a", true);
  assert.equal(state.visible("dm:a", true), "4");
  state.acknowledge("dm:a", "4");
  state.acknowledge("dm:a", "2");
  assert.equal(state.visible("dm:a", true), null);
});

test("bots count as unread; leaving/deleting a group drops its counts and receipt watermark", () => {
  const state = new ReadState();
  state.receive(message("1", "Pirate", "group:g"), "b", null, true);
  state.receive(message("2", "a"), "b", null, true);
  assert.deepEqual(state.counts(), { "group:g": 1, "dm:a": 1 });
  state.visible("group:g", true);
  state.acknowledge("group:g", "1");
  state.retainGroups(new Set());
  assert.deepEqual(state.counts(), { "dm:a": 1 });
  assert.equal(state.visible("group:g", true), null);
  state.receive(message("3", "Pirate", "group:g"), "b", null, false);
  assert.equal(state.visible("group:g", true), "3");
  state.reset();
  assert.deepEqual(state.counts(), {});
  assert.equal(state.visible("dm:a", true), null);
});

test("history is observed without adding unread; u64 ids stay exact beyond JavaScript number precision", () => {
  const state = new ReadState();
  state.observeHistory("dm:a", [message("18446744073709551614")]);
  assert.deepEqual(state.counts(), {});
  assert.equal(state.visible("dm:a", true), "18446744073709551614");
  state.acknowledge("dm:a", "18446744073709551614");
  state.receive(message("18446744073709551615"), "b", null, true);
  assert.equal(state.visible("dm:a", true), "18446744073709551615");
});

test("a receipt marks only existing own messages through its exact id, once per reader", () => {
  const own = [
    message("1", "me"),
    message("2", "me"),
    message("3", "me"),
    message("4", "a"),
  ];
  const receipt = { roomId: "dm:a", by: "a", upto: "2" };
  let messages = applyReceipt(own, receipt, "me");
  messages = applyReceipt(messages, receipt, "me");
  assert.deepEqual(
    messages.map((m) => m.readers),
    [["a"], ["a"], [], []],
  );
  assert.equal(newestSeen(messages, "me")?.id, "2");
  assert.deepEqual(
    applyReceipt(messages, { ...receipt, by: "me" }, "me"),
    messages,
  );
  assert.deepEqual(
    applyReceipt(messages, { ...receipt, roomId: "other" }, "me"),
    messages,
  );
  messages = applyReceipt(messages, { ...receipt, by: "c" }, "me");
  assert.deepEqual(newestSeen(messages, "me")?.readers, ["a", "c"]);
});

test("merging a room load with live events preserves receipts and orders messages by id", () => {
  const old = { ...message("9", "me"), readers: ["a"] };
  const updated = { ...message("9", "me"), readers: ["b"] };
  const merged = mergeMessages([old], [message("10", "a"), updated]);
  assert.deepEqual(
    merged.map((m) => m.id),
    ["9", "10"],
  );
  assert.deepEqual(merged[0].readers, ["a", "b"]);
});

test("a receipt arriving while history loads is not lost with an older snapshot", () => {
  const history = [message("1", "me"), message("2", "me")];
  const merged = mergeMessages(
    history,
    [],
    [{ roomId: "dm:a", by: "a", upto: "1" }],
    "me",
  );
  assert.deepEqual(
    merged.map((m) => m.readers),
    [["a"], []],
  );
});
