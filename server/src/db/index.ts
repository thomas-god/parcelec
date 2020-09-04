import { Pool, QueryResult } from "pg";

const databaseConfig = {
  connectionString: "postgres://docker:docker@db:5432/docker",
};
const pool = new Pool(databaseConfig);

export default {
  query: (text: string, params: any): Promise<QueryResult> =>
    pool.query(text, params),
};
