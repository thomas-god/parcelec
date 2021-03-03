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

const route = '/session/';
const openSessionID = 'a19bc943-4599-4782-a650-806b015f209a';
const runningSessionID = '04c92e13-a42f-4381-aa67-94875798082e';
const closedSessionID = 'a196d524-ab6d-4adc-bcab-42e96f5ce547';

describe('Registering a new user', () => {
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

  it('should register a new user', async () => {
    const res = await chai
      .request(server)
      .put(route + openSessionID + '/user')
      .query({ username: 'toto' });

    expect(res.status).to.eql(201);
    expect(uuidValidate(res.body.userId)).to.be.true;
  });

  it('should not register a new user to running session', async () => {
    const res = await chai
      .request(server)
      .put(route + runningSessionID + '/user')
      .query({ username: 'toto' });

    expect(res.status).to.eql(400);
    expect(res.body.message).to.eql('Session is not open for registration.');
  });

  it('should not register a new user to closed session', async () => {
    const res = await chai
      .request(server)
      .put(route + closedSessionID + '/user')
      .query({ username: 'toto' });

    expect(res.status).to.eql(400);
    expect(res.body.message).to.eql('Session is not open for registration.');
  });

  it('should not register a new user to sessions that does not exist', async () => {
    const id = uuid();
    const res = await chai
      .request(server)
      .put(route + id + '/user')
      .query({ username: 'toto' });

    expect(res.status).to.eql(400);
    expect(res.body.message).to.eql(`Cannot find a session with ID ${id}.`);
  });

  it('should not register a new user with missing username', async () => {
    const id = uuid();
    const res = await chai
      .request(server)
      .put(route + id + '/user')
      .query({});

    expect(res.status).to.eql(400);
    expect(res.body.message).to.eql(
      "request.query should have required property 'username'"
    );
  });

  it('should not register a new user already existing username', async () => {
    await chai
      .request(server)
      .put(route + openSessionID + '/user')
      .query({ username: 'tutu' });
    const res = await chai
      .request(server)
      .put(route + openSessionID + '/user')
      .query({ username: 'tutu' });

    expect(res.status).to.eql(400);
    expect(res.body.message).to.eql('Could not create user.');
  });
});
