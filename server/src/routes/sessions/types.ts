export interface Session {
  id: string;
  name: string;
  status: "open" | "running" | "closed";
}
