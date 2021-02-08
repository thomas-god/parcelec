import { Dependencies } from '../../di.context'

export class BidsDAO {
  private db: Dependencies["db"]

  constructor({ db }: { db: Dependencies["db"]}) {
    this.db = db;
  }
}