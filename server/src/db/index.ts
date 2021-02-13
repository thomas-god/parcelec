import { Pool, QueryResult } from "pg";

const databaseConfig = {
  connectionString: process.env.DATABASE_URL,
};
const pool = new Pool(databaseConfig);

pool.on("error", (err, client) => {
  console.log("#########################################");
  console.log("######## Error with the database ########");
  console.log(client);
  console.log(err);
});

pool.on("connect", (client) => {
  console.log("Connected to database")
})

export default {
  query: (text: string, params: any): Promise<QueryResult> =>
    pool.query(text, params),
};
