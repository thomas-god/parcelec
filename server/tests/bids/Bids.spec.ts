import chai from 'chai';
import chaiHttp from 'chai-http';
import getContext from '../../src/di.context'
import { createServer } from '../../src/server'
import { v4 as uuid, validate as validateUuid } from 'uuid'

const context = getContext()
// Can mock part of context
const app = createServer(context)
const server = app.listen(3000, (err) => {});

chai.use(chaiHttp);
const expect = chai.expect;

const url = `/beta/session/${uuid()}/user/${uuid()}/bid`;

describe("Bids route", () => {

  afterAll(async () => {
    await server.close();
  });

  it("should succes", async () => {
    const res = await chai.request(server).post(url)
    .send({ bid: { type: "buy", volume_mwh: 10, price_eur_per_mwh: 50 } })

    expect(res.status).to.eql(201)
    expect(validateUuid(res.body.id)).to.be.true

  })

  it("should failed with missing volume", async () => {
    const res = await chai.request(server).post(url)
    .send({ bid: { type: "buy", price_eur_per_mwh: 50 } })

    expect(res.status).to.eql(400)
    expect(res.body.message).to.eql("request.body.bid should have required property 'volume_mwh'")
  })

  it("should failed with non number volume", async () => {
    const res = await chai.request(server).post(url)
    .send({ bid: { type: "buy", price_eur_per_mwh: 4, volume_mwh: "toto" } })

    expect(res.status).to.eql(400)
    expect(res.body.message).to.eql("request.body.bid.volume_mwh should be integer")
  })

  it("should failed with missing price", async () => {
    const res = await chai.request(server).post(url)
    .send({ bid: { type: "buy", volume_mwh: 50 } })

    expect(res.status).to.eql(400)
    expect(res.body.message).to.eql("request.body.bid should have required property 'price_eur_per_mwh'")
  })

  it("should failed with non number price", async () => {
    const res = await chai.request(server).post(url)
    .send({ bid: { type: "buy", price_eur_per_mwh: "toto", volume_mwh: 50 } })

    expect(res.status).to.eql(400)
    expect(res.body.message).to.eql("request.body.bid.price_eur_per_mwh should be integer")
  })

  it("should failed with missing bid type", async () => {
    const res = await chai.request(server).post(url)
    .send({ bid: { price_eur_per_mwh: 4, volume_mwh: 50 } })

    expect(res.status).to.eql(400)
    expect(res.body.message).to.eql("request.body.bid should have required property 'type'")
  })

  it("should failed with wrong bid type", async () => {
    const res = await chai.request(server).post(url)
    .send({ bid: { type: "toto", price_eur_per_mwh: 4, volume_mwh: 50 } })

    expect(res.status).to.eql(400)
    expect(res.body.message).to.eql("request.body.bid.type should be equal to one of the allowed values: sell, buy")
  })
})