import { promises as fs } from 'fs';
import { resolve } from 'path';
import createContainer, { Dependencies } from '../src/di.context';

const container = createContainer();

export async function setUpDB(): Promise<void> {
  const db = container.resolve('db') as Dependencies['db'];
  const files = await fs.readdir(resolve(__dirname, './sqls'));

  for (const file of files) {
    const query = (
      await fs.readFile(resolve(__dirname, './sqls', file))
    ).toString();
    await db.execute(query, []);
  }
}

export async function clearDB(): Promise<void> {
  const db = container.resolve('db') as Dependencies['db'];
  await db.execute(
    `
    TRUNCATE t_sessions CASCADE;
    TRUNCATE t_users CASCADE;
    TRUNCATE t_bids CASCADE;
  `,
    []
  );
}
