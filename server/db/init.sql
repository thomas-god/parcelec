CREATE TABLE auctions
(
  id UUID PRIMARY KEY,
  name VARCHAR(50) UNIQUE,
  status text
);

CREATE TABLE auctions_steps
(
  auction_id UUID REFERENCES auctions (id),
  step_no INT,
  status VARCHAR(10) CHECK (status in ('open', 'close')),
  clearing_price REAL DEFAULT null,
  PRIMARY KEY (auction_id, step_no)
);

CREATE TABLE users
(
  id UUID PRIMARY KEY,
  name VARCHAR(50),
  auction_id UUID REFERENCES auctions (id)
);

CREATE TABLE bids
(
  user_id UUID REFERENCES users (id),
  auction_id UUID,
  auction_step_no INT,
  bid_value REAL,
  FOREIGN KEY (auction_id, auction_step_no) REFERENCES auctions_steps (auction_id, step_no)
);