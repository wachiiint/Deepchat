export type Group = {
  id: string;
  name: string;
  creator: string;
  members: string[];
  bots: string[];
};

export type Message = {
  id: string;
  roomId: string;
  sender: string;
  text: string;
  timestamp: number;
  peer: string | null;
  readers: string[];
};

// Someone typed in a room. The room id matches `Message.roomId`.
export type TypingEvent = {
  roomId: string;
  from: string;
};

// A membership change seen while a group was open. Lives only in the frontend.
export type RoomEvent = {
  id: string;
  roomId: string;
  kind: "joined" | "left";
  user: string;
  timestamp: number;
};

export type ConnectResult = {
  name: string;
  users: string[];
  groups: Group[];
};

export type RoomKind = "private" | "group";

export type ActiveRoom = {
  id: string;
  kind: RoomKind;
  title: string;
};

export type ReadReceipt = { roomId: string; by: string; upto: string };
