import { v4 as uuid } from 'uuid';
import { Dependencies } from '../../di.context';
import { Bid } from '../types';

import { BidTypes } from './types';

export class BidsService {
  private BidsDAO: Dependencies['BidsDAO'];
  private UsersService: Dependencies['UsersService'];
  private SessionsService: Dependencies['SessionsService'];

  constructor({
    BidsDAO,
    UsersService,
    SessionsService,
  }: {
    BidsDAO: Dependencies['BidsDAO'];
    UsersService: Dependencies['UsersService'];
    SessionsService: Dependencies['SessionsService'];
  }) {
    this.BidsDAO = BidsDAO;
    this.UsersService = UsersService;
    this.SessionsService = SessionsService;
  }

  /**
   * Post a new bid to a running auction
   * @param sessionId Session UUID
   * @param userId User UUID
   * @param body Bid to post
   */
  async postUserBid(
    sessionId: string,
    userId: string,
    type: BidTypes,
    volume: number,
    price: number
  ): Promise<string> {
    await this.SessionsService.getSessionIfRunning(sessionId);
    await this.UsersService.getUser(sessionId, userId);

    const bid = await this.BidsDAO.createBid(
      sessionId,
      userId,
      type,
      volume,
      price
    );
    return bid.id;
  }
}
