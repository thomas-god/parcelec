import { Server } from 'http';
import { validate as uuidValidate } from 'uuid';
import chai from 'chai';
import chaiHttp from 'chai-http';
import getContext from '../../src/di.context';
import { createServer } from '../../src/server';
import { clearDB, setUpDB } from '../setupDB';

const expect = chai.expect;

chai.use(chaiHttp);

let server: Server;

const openSessionID = 'a19bc943-4599-4782-a650-806b015f209a';
const runningSessionID = '04c92e13-a42f-4381-aa67-94875798082e';

describe('Create a new session', () => {
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

  it('should get the id of the last phase of the session', async () => {
    const res = await chai
      .request(server)
      .get(`/session/${runningSessionID}/phase`);

    const expected = {
      phase_no: 1,
      status: 'open',
    };

    expect(res.status).to.be.eql(200);
    expect(res.body).to.be.deep.eq(expected);
  });

  it('should have empty for session with no phase', async () => {
    const res = await chai
      .request(server)
      .get(`/session/${openSessionID}/phase`);

    const expected = {};

    expect(res.body).to.be.deep.eq(expected);
    expect(res.status).to.be.eql(204);
  });
});
