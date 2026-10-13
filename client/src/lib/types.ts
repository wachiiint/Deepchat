export type Group = {
  id: string;
  name: string;
  members: string[];
};

export type Message = {
  id: string;
  roomId: string;
  sender: string;
  text: string;
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
