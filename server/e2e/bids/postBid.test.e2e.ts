import { Server } from 'http';
import { v4 as uuid, validate as uuidValidate } from 'uuid';
import chai from 'chai';
import chaiHttp from 'chai-http';
import getContext from '../../src/di.context';
import { createServer } from '../../src/server';
import { clearDB, setUpDB } from '../setupDB';

const expect = chai.expect;

chai.use(chaiHttp);

let server: Server;

const runningSessionID = '04c92e13-a42f-4381-aa67-94875798082e';
const runningUserID = 'a5418fc9-9daa-4716-badc-c3eff79f3fc9';
const notRunningSessionID = 'a19bc943-4599-4782-a650-806b015f209a';

describe('Post a new bid', () => {
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

  it('should successfully post a bid', async () => {
    const res = await chai
      .request(server)
      .post(`/session/${runningSessionID}/user/${runningUserID}/bid`)
      .send({ bid: { type: 'buy', price: 50, volume: 10 } });

    expect(res.status).to.eql(201);
    expect(uuidValidate(res.body.id)).to.be.true;
  });

  it('should failed with missing volume', async () => {
    const res = await chai
      .request(server)
      .post(`/session/${runningSessionID}/user/${runningUserID}/bid`)
      .send({ bid: { type: 'buy', price: 50 } });

    expect(res.status).to.eql(400);
    expect(res.body.message).to.eql(
      "request.body.bid should have required property 'volume'"
    );
  });

  it('should failed with non number volume', async () => {
    const res = await chai
      .request(server)
      .post(`/session/${runningSessionID}/user/${runningUserID}/bid`)
      .send({ bid: { type: 'buy', price: 4, volume: 'toto' } });

    expect(res.status).to.eql(400);
    expect(res.body.message).to.eql(
      'request.body.bid.volume should be integer'
    );
  });

  it('should failed with missing price', async () => {
    const res = await chai
      .request(server)
      .post(`/session/${runningSessionID}/user/${runningUserID}/bid`)
      .send({ bid: { type: 'buy', volume: 50 } });

    expect(res.status).to.eql(400);
    expect(res.body.message).to.eql(
      "request.body.bid should have required property 'price'"
    );
  });

  it('should failed with non number price', async () => {
    const res = await chai
      .request(server)
      .post(`/session/${runningSessionID}/user/${runningUserID}/bid`)
      .send({
        bid: { type: 'buy', price: 'toto', volume: 50 },
      });

    expect(res.status).to.eql(400);
    expect(res.body.message).to.eql('request.body.bid.price should be integer');
  });

  it('should failed with missing bid type', async () => {
    const res = await chai
      .request(server)
      .post(`/session/${runningSessionID}/user/${runningUserID}/bid`)
      .send({ bid: { price: 4, volume: 50 } });

    expect(res.status).to.eql(400);
    expect(res.body.message).to.eql(
      "request.body.bid should have required property 'type'"
    );
  });

  it('should failed with wrong bid type', async () => {
    const res = await chai
      .request(server)
      .post(`/session/${runningSessionID}/user/${runningUserID}/bid`)
      .send({ bid: { type: 'toto', price: 4, volume: 50 } });

    expect(res.status).to.eql(400);
    expect(res.body.message).to.eql(
      'request.body.bid.type should be equal to one of the allowed values: sell, buy'
    );
  });

  it('should failed with non existing session', async () => {
    const id = uuid();
    const res = await chai
      .request(server)
      .post(`/session/${id}/user/${runningUserID}/bid`)
      .send({ bid: { type: 'sell', price: 4, volume: 50 } });

    expect(res.status).to.eql(400);
    expect(res.body.message).to.eql(`Cannot find a session with ID ${id}.`);
  });

  it('should failed with a not running session', async () => {
    const res = await chai
      .request(server)
      .post(`/session/${notRunningSessionID}/user/${runningUserID}/bid`)
      .send({ bid: { type: 'sell', price: 4, volume: 50 } });

    expect(res.status).to.eql(400);
    expect(res.body.message).to.eql(
      `Session ${notRunningSessionID} is not running.`
    );
  });

  it('should failed with not existing user', async () => {
    const id = uuid();
    const res = await chai
      .request(server)
      .post(`/session/${runningSessionID}/user/${id}/bid`)
      .send({ bid: { type: 'sell', price: 4, volume: 50 } });

    expect(res.status).to.eql(400);
    expect(res.body.message).to.eql(`Cannot find a user with ID ${id}.`);
  });
});
