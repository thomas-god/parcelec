CREATE TABLE auctions
(
  id UUID PRIMARY KEY,
  name TEXT UNIQUE,
  status TEXT
);

CREATE TABLE auctions_steps
(
  auction_id UUID REFERENCES auctions (id),
  step_no INT,
  status TEXT CHECK (status in ('open', 'closed')),
  clearing_price REAL DEFAULT null,
  PRIMARY KEY (auction_id, step_no)
);

CREATE TABLE users
(
  id UUID PRIMARY KEY,
  name TEXT,
  ready BOOLEAN DEFAULT FALSE,
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