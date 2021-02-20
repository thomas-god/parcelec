import { Pool, QueryResult, QueryResultRow } from "pg";

function getConnectionString(): string {
  const sslMode =
    process.env.NODE_ENV === "production" ? "no-verify" : "disable";
  return process.env.DATABASE_URL + "?sslmode=" + sslMode;
}

export type DBResult = {
  rows: QueryResultRow;
  count: number;
};
export class Database {
  private pool: Pool;

  constructor() {
    this.pool = new Pool({ connectionString: getConnectionString() });
  }

  /**
   * Execute a query against the database, wrapped into a transaction
   * @param query query string
   * @param subs parameters to substitute in the query
   */
  async execute(query: string, subs: any[]): Promise<DBResult> {
    const client = await this.pool.connect();
    let result: QueryResult | null = null;
    try {
      await client.query("BEGIN");
      result = await client.query(query, subs);
      await client.query("COMMIT");
    } catch (e) {
      client.query("ROLLBACK");
      throw e;
    } finally {
      client.release();
    }
    return { rows: result.rows, count: result.rowCount };
  }
}

// OLD
let databaseConfig: any = {
  connectionString: process.env.DATABASE_URL,
};
const pool = new Pool(databaseConfig);

pool.on("error", (err, client) => {
  console.log("#########################################");
  console.log("######## Error with the database ########");
  console.log(client);
  console.log(err);
});

export default {
  query: (text: string, params: any): Promise<QueryResult> =>
    pool.query(text, params),
};
// OLD
