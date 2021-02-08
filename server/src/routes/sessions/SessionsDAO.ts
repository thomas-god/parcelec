import { container, Dependencies } from '../../di.context'

export class SessionsDAO {
  private db: Dependencies["db"]

  constructor() {
    this.db = container.resolve("db")
  }
}