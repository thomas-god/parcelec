import { Pool, QueryResult } from "pg";

const databaseConfig = {
  connectionString: "postgres://docker:docker@db:5432/docker",
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
