import { v4 as uuid, validate as uuidValidate } from "uuid";
import chai from 'chai';
import chaiHttp from 'chai-http';
import getContext from "../../src/di.context";
import { createServer } from "../../src/server";
import { clearDB, setUpDB } from '../setupDB'


const expect = chai.expect;

chai.use(chaiHttp);

let server;
const route = '/session';

describe("Create a new session", () => {
  before(async () => {
    const context = getContext();
    const app = await createServer(context);
    server = app.listen(3000, (err) => { });
    await setUpDB()
  })

  after(async () => {
    server.close()
    await clearDB()
  })

  it("should get a list of users", async () => {
    const res = await chai.request(server).get(`/session/a19bc943-4599-4782-a650-806b015f209a/users`)

    expect(res.status).to.eql(200)
    expect(res.body).to.deep.equal([
      {
        name: 'User toto',
        isReady: false
      }
    ])
  })

  it("should get a empty list if no users", async () => {
    const res = await chai.request(server).get(`/session/04c92e13-a42f-4381-aa67-94875798082e/users`)

    expect(res.status).to.eql(200)
    expect(res.body).to.deep.equal([])
  })

  it("should get an error if session does not exists", async () => {
    const id = uuid();
    const res = await chai.request(server).get(`/session/${id}/users`)

    expect(res.status).to.eql(400)
    expect(res.body.message).to.eql(`Cannot find a session with ID ${id}.`);
  })

})