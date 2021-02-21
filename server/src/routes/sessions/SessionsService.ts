import { Dependencies } from "../../di.context";
import { Session, SessionStatus } from "./types";

export class SessionsService {
  private SessionsDAO: Dependencies["SessionsDAO"];

  constructor({ SessionsDAO }: { SessionsDAO: Dependencies["SessionsDAO"] }) {
    this.SessionsDAO = SessionsDAO;
  }

  async createSession(sessionName: string): Promise<string> {
    const session: Session = await this.SessionsDAO.createSession(sessionName);
    return session.id;
  }

  async getSessionList(
    status: SessionStatus
  ): Promise<Omit<Session, "status">[]> {
    return (await this.SessionsDAO.getSessionsList(status)).map(
      (session: Session) => {
        return <Omit<Session, "status">>{
          id: session.id,
          name: session.name,
        };
      }
    );
  }
}
