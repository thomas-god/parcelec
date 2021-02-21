export interface Session {
  id: string;
  name: string;
  status: "open" | "running" | "closed";
}

export enum SessionStatus {
  open = "open",
  running = "running",
  closed = "closed",
}
