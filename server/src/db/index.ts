import { Pool, QueryResult } from "pg";

const databaseConfig = {
  connectionString: process.env.DATABASE_URL,
  ssl: true
};
const pool = new Pool(databaseConfig);

pool.on("error", (err, client) => {
  console.log("#########################################");
  console.log("######## Error with the database ########");
  console.log(client);
  console.log(err);
});

pool.on("acquire", (client) => {
  console.log("Connected to database")
})

export async function buildDefaultDB() {
  await pool.query(`
    CREATE TABLE films (
      code        char(5) CONSTRAINT firstkey PRIMARY KEY,
      title       varchar(40) NOT NULL,
      did         integer NOT NULL,
      date_prod   date,
      kind        varchar(10),
      len         interval hour to minute
    );
  `);
  await pool.query(`
    INSERT INTO films VALUES
      ('UA502', 'Bananas', 105, '1971-07-13', 'Comedy', '82 minutes');
  `);
}

export default {
  query: (text: string, params: any): Promise<QueryResult> =>
    pool.query(text, params),
};
