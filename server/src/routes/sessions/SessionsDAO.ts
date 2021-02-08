import { Database } from '../../db'
import { Dependencies } from '../../di.context'

export class SessionsDAO {
  private db: Dependencies["db"]

  constructor({ db }: { db: Database}) {
    this.db = db
  }
}