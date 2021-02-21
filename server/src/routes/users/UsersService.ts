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
    if (session === undefined) {
      throw new Error(`Cannot find a session with ID ${sessionId}.`);
    }
    if (session.status !== "open") {
      throw new Error(`Session is not open for registration.`);
    }

    try {
      const user = await this.UsersDAO.createUser(sessionId, username);
      return user.id;
    } catch (err) {
      throw new Error(`Could not create user.`);
    }
  }

  async markUserReady(sessionId: string, userId: string): Promise<void> {
    const session = await this.SessionsDAO.getSession(sessionId);
    if (session === undefined) {
      throw new Error(`Cannot find a session with ID ${sessionId}.`);
    }

    const user = await this.UsersDAO.getUser(sessionId, userId);
    if (user === undefined) {
      throw new Error(`Cannot find a user with ID ${userId}.`);
    }

    await this.UsersDAO.markUserReady(sessionId, userId);
  }
}
