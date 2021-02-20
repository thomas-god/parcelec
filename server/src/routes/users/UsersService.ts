import { Dependencies } from "../../di.context";
import { User } from "./types";

export class UsersService {
  private UsersDAO: Dependencies["UsersDAO"];
  private SessionsDAO: Dependencies["SessionsDAO"];

  constructor({
    UsersDAO,
    SessionsDAO,
  }: {
    UsersDAO: Dependencies["UsersDAO"];
    SessionsDAO: Dependencies["SessionsDAO"];
  }) {
    this.UsersDAO = UsersDAO;
    this.SessionsDAO = SessionsDAO;
  }

  async registerUser(sessionId: string, username: string): Promise<string> {
    const session = await this.SessionsDAO.getSession(sessionId);
    if (session.length !== 1) {
      throw new Error(`Cannot find a session with ID ${sessionId}.`);
    }
    if (session[0].status !== "open") {
      throw new Error(`Session is not open for registration.`);
    }

    try {
      const user = await this.UsersDAO.createUser(sessionId, username);
      return user.id;
    } catch (err) {
      throw new Error(`Could not create user.`);
    }
  }
}
