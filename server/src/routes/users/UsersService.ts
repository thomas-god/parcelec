import { Dependencies } from '../../di.context';
import { User } from './types';

export class UsersService {
  private UsersDAO: Dependencies['UsersDAO'];
  private SessionsService: Dependencies['SessionsService'];

  constructor({
    UsersDAO,
    SessionsService,
  }: {
    UsersDAO: Dependencies['UsersDAO'];
    SessionsService: Dependencies['SessionsService'];
  }) {
    this.UsersDAO = UsersDAO;
    this.SessionsService = SessionsService;
  }

  async registerUser(sessionId: string, username: string): Promise<string> {
    await this.SessionsService.getSessionIfOpen(sessionId);

    try {
      const user = await this.UsersDAO.createUser(sessionId, username);
      return user.id;
    } catch (err) {
      throw new Error(`Could not create user.`);
    }
  }

  async markUserReady(sessionId: string, userId: string): Promise<void> {
    await this.SessionsService.getSession(sessionId);
    await this.getUser(sessionId, userId);

    await this.UsersDAO.markUserReady(sessionId, userId);
  }

  async getSessionUsers(
    sessionId: string
  ): Promise<Pick<User, 'name' | 'isReady'>[]> {
    await this.SessionsService.getSession(sessionId);

    return (await this.UsersDAO.getUsers(sessionId)).map((user: User) => {
      return <Pick<User, 'name' | 'isReady'>>{
        name: user.name,
        isReady: user.isReady,
      };
    });
  }

  async getUser(sessionId: string, userId: string): Promise<User> {
    const user = await this.UsersDAO.getUser(sessionId, userId);
    if (user === undefined) {
      throw new Error(`Cannot find a user with ID ${userId}.`);
    }
    return user;
  }
}
