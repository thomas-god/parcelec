/// Price at wich a player's excess production will be bought (in €/MWh)
pub const POSITIVE_IMBALANCE_COST: isize = 15;

/// Price a player will have to pay for production deficit (in €/MWh)
pub const NEGATIVE_IMBALANCE_COST: isize = 100;

/// Price of the buy offer that will always be on the market (in ct€/MWh)
pub const MARKET_EXTREME_BUY_OFFER_PRICE: isize = 25_00;

/// Price of the sell offer that will always be on the market (in ct€/MWh)
pub const MARKET_EXTREME_SELL_OFFER_PRICE: isize = 90_00;

/// Volume of the buy/sell offer that will always be on the market (in MWh)
pub const MARKET_EXTREME_OFFERS_VOLUME: usize = 250;
