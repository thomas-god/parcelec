/// Price at wich a player's excess production will be bought (in €/MWh)
pub const POSITIVE_IMBALANCE_COST: i32 = 15;

/// Price a player will have to pay for production deficit (in €/MWh)
pub const NEGATIVE_IMBALANCE_COST: i32 = 100;

/// Price of the buy offer that will always be on the market (in €/MWh)
pub const MARKET_EXTREME_BUY_OFFER_PRICE: i32 = 25;

/// Price of the sell offer that will always be on the market (in €/MWh)
pub const MARKET_EXTREME_SELL_OFFER_PRICE: i32 = 90;

/// Volume of the buy/sell offer that will always be on the market (in MWh)
pub const MARKET_EXTREME_OFFERS_VOLUME: i32 = 250;

/// Base deviation when forecasting a value (in MW)
pub const FORECAST_BASE_DEVIATION: i32 = 100;

/// Base for setpoints, i.e. setpoints should be a multiple of these value (in MW)
pub const SETPOINT_BASE_VALUE: i32 = 25;

/// Default duration for market and stacks periods in seconds
pub const DEFAULT_PERIOD_DURATION_SECONDS: u64 = 120;
