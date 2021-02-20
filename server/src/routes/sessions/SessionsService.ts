import { Dependencies } from "../../di.context";
import { Session } from "./types";

export class SessionsService {
  private SessionsDAO: Dependencies["SessionsDAO"];

  constructor({ SessionsDAO }: { SessionsDAO: Dependencies["SessionsDAO"] }) {
    this.SessionsDAO = SessionsDAO;
  }

  async createSession(sessionName: string): Promise<string> {
    const session: Session = await this.SessionsDAO.createSession(sessionName);
    return session.id;
  }
}
