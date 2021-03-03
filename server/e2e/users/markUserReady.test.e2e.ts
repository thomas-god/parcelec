import { Server } from 'http';
import { v4 as uuid, validate as uuidValidate } from 'uuid';
import chai from 'chai';
import chaiHttp from 'chai-http';
import { createServer } from '../../src/server';
import getContext from '../../src/di.context';

import { clearDB, setUpDB } from '../setupDB';

const expect = chai.expect;
chai.use(chaiHttp);

let server: Server;

const openSessionID = 'a19bc943-4599-4782-a650-806b015f209a';
const userID = 'c7c273c1-574e-4f88-a0a6-38e0ab3b23ab';

function getRoute(sessionId: string, userId: string): string {
  return `/session/${sessionId}/user/${userId}/ready`;
}

describe('Mark a user ready', () => {
  before(async () => {
    const context = getContext();
    const app = await createServer(context);
    server = app.listen(3000, () => {});
    await setUpDB();
  });

  after(async () => {
    server.close();
    await clearDB();
  });

  it('should mark user ready', async () => {
    const res = await chai.request(server).put(getRoute(openSessionID, userID));
    expect(res.status).to.eql(200);
  });

  it('should be idempotent', async () => {
    await chai.request(server).put(getRoute(openSessionID, userID));
    const res = await chai.request(server).put(getRoute(openSessionID, userID));
    expect(res.status).to.eql(200);
  });

  it('should fail to non existing session', async () => {
    const id = uuid();
    const res = await chai.request(server).put(getRoute(id, userID));
    expect(res.status).to.eql(400);
    expect(res.body.message).to.eql(`Cannot find a session with ID ${id}.`);
  });

  it('should fail to non existing suer', async () => {
    const id = uuid();
    const res = await chai.request(server).put(getRoute(openSessionID, id));
    expect(res.status).to.eql(400);
    expect(res.body.message).to.eql(`Cannot find a user with ID ${id}.`);
  });
});
