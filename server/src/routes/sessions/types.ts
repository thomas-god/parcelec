export interface Session {
  id: string;
  name: string;
  status: SessionStatus;
}

export enum SessionStatus {
  open = "open",
  running = "running",
  closed = "closed",
}
