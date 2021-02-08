import { v4 as uuid } from 'uuid';
import { Dependencies } from '../../di.context'
import { Bid } from '../types';

export interface BidInput {
  type: "sell" | "buy";
  volume_mwh: number;
  price_eur_per_mwh: number;
}

export class BidsService {
    private BidsDAO : Dependencies["BidsDAO"];

    constructor({ BidsDAO }: { BidsDAO: Dependencies["BidsDAO"] }) {
        this.BidsDAO = BidsDAO;
    }

    async postUserBid(sessionId: string, userId: string, body: BidInput): Promise<Bid> {
      return {
        id: uuid(),
        user_id: userId,
        session_id: sessionId,
        phase_no: 0,
        type: body.type,
        volume_mwh: body.volume_mwh,
        price_eur_per_mwh: body.volume_mwh
      }
    }
}