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

const route = '/session';

describe('Get open sessions list', () => {
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

  it('should get a list of open sessions', async () => {
    const res = await chai.request(server).get('/sessions');

    expect(res.status).to.eql(200);
    expect(res.body).to.deep.equal([
      {
        id: 'a19bc943-4599-4782-a650-806b015f209a',
        name: 'Open session',
      },
    ]);
  });
});
