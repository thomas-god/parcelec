import { Dependencies } from '../../di.context';
import { Session, SessionStatus } from './types';

export class SessionsService {
  private SessionsDAO: Dependencies['SessionsDAO'];

  constructor({ SessionsDAO }: { SessionsDAO: Dependencies['SessionsDAO'] }) {
    this.SessionsDAO = SessionsDAO;
  }

  async createSession(sessionName: string): Promise<string> {
    const session: Session = await this.SessionsDAO.createSession(sessionName);
    return session.id;
  }

  async getSessionList(
    status: SessionStatus
  ): Promise<Omit<Session, 'status'>[]> {
    return (await this.SessionsDAO.getSessionsList(status)).map(
      (session: Session) => {
        return <Omit<Session, 'status'>>{
          id: session.id,
          name: session.name,
        };
      }
    );
  }

  async getSession(sessionID: string): Promise<Session> {
    const session = await this.SessionsDAO.getSession(sessionID);
    if (session === undefined) {
      throw new Error(`Cannot find a session with ID ${sessionID}.`);
    }
    return session;
  }

  async getSessionIfOpen(sessionID: string): Promise<Session> {
    const session = await this.getSession(sessionID);
    if (session.status !== 'open') {
      throw new Error(`Session ${sessionID} is not open for registration.`);
    }
    return session;
  }

  async getSessionIfRunning(sessionID: string): Promise<Session> {
    const session = await this.getSession(sessionID);
    if (session.status !== 'running') {
      throw new Error(`Session ${sessionID} is not running.`);
    }
    return session;
  }
}
