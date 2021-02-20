import { validate as uuidValidate } from "uuid";
import chai from 'chai';
import chaiHttp from 'chai-http';
import getContext from "../../src/di.context";
import { createServer } from "../../src/server";


const expect = chai.expect;

chai.use(chaiHttp);

let server;
const route = '/session';

describe("Create a new session", () => {
  before(async () => {
    const context = getContext();
    const app = await createServer(context);
    server = app.listen(3000, (err) => { });
  })

  after(async () => {
    server.close()
  })

  it("should create a new session", async () => {
    const res = await chai.request(server).put(route).query({ sessionName: 'toto' });

    expect(res.status).to.be.eql(201);
    expect(uuidValidate(res.body.sessionId)).to.be.true;
  })

  it("should fail with no session name", async () => {
    const res = await chai.request(server).put(route).query({});

    expect(res.status).to.be.eql(400);
    expect(res.body.message).to.eql('request.query should have required property \'sessionName\'')
  })
})