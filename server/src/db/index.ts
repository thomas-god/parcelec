import "dotenv/config";
import { Pool, QueryResult } from "pg";

const databaseConfig = {
  connectionString: process.env.DATABASE_URL,
};
const pool = new Pool(databaseConfig);
console.log(process.env.DATABASE_URL);
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
