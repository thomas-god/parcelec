use std::{collections::HashMap, fmt::Debug, ops::Add};

use serde::Serialize;

use crate::{
    constants::{NEGATIVE_IMBALANCE_COST, POSITIVE_IMBALANCE_COST},
    market::{
        Direction,
        order_book::{Trade, TradeLeg},
    },
    plants::{Output, PlantId, PlantOutput, StackDispatchResults},
    player::PlayerId,
    utils::units::{Energy, EnergyCost, Money, NO_POWER, Power, TIMESTEP, ZERO_ENERGY},
};

use super::delivery_period::DeliveryPeriodId;

#[derive(Debug, PartialEq, Default, Clone, Serialize)]
pub struct PlayerScore {
    pub balance: Power,
    pub pnl: Money,
    pub imbalance_cost: Money,
}

impl Add<PlayerScore> for PlayerScore {
    type Output = PlayerScore;
    fn add(self, rhs: PlayerScore) -> Self::Output {
        PlayerScore {
            balance: self.balance + rhs.balance,
            pnl: self.pnl + rhs.pnl,
            imbalance_cost: self.imbalance_cost + rhs.imbalance_cost,
        }
    }
}

impl Add<&PlantOutput> for PlayerScore {
    type Output = PlayerScore;
    fn add(self, rhs: &PlantOutput) -> PlayerScore {
        PlayerScore {
            balance: self.balance + rhs.setpoint,
            pnl: self.pnl + rhs.cost,
            imbalance_cost: self.imbalance_cost,
        }
    }
}

impl Add<TradeLeg> for PlayerScore {
    type Output = PlayerScore;
    fn add(self, rhs: TradeLeg) -> Self::Output {
        let trade_volume = if rhs.direction == Direction::Buy {
            rhs.volume
        } else {
            -rhs.volume
        };
        let trade_pnl = if rhs.direction == Direction::Buy {
            -rhs.price * rhs.volume
        } else {
            rhs.price * rhs.volume
        };
        PlayerScore {
            balance: self.balance + trade_volume / TIMESTEP,
            pnl: self.pnl + trade_pnl,
            imbalance_cost: self.imbalance_cost,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ScoreDetails {
    pub volume: Energy,
    pub pnl: Money,
}

impl From<&Output> for ScoreDetails {
    fn from(value: &Output) -> Self {
        ScoreDetails {
            volume: *value.volume(),
            pnl: *value.money(),
        }
    }
}

impl Default for ScoreDetails {
    fn default() -> Self {
        Self {
            volume: Energy::from(0),
            pnl: Money::from(0),
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct PlayerDetailedScore {
    pub consumers: ScoreDetails,
    pub renewables: ScoreDetails,
    pub gas: ScoreDetails,
    pub nuclear: ScoreDetails,
    pub battery_discharge: ScoreDetails,
    pub battery_charge: ScoreDetails,
    pub market_bought: ScoreDetails,
    pub market_sold: ScoreDetails,
    pub imbalance: ScoreDetails,
}

impl PlayerDetailedScore {
    pub fn position(&self) -> Energy {
        self.consumers.volume
            + self.renewables.volume
            + self.gas.volume
            + self.nuclear.volume
            + self.battery_discharge.volume
            + self.battery_charge.volume
            + self.market_bought.volume
            + self.market_sold.volume
            + self.imbalance.volume
    }
    pub fn pnl(&self) -> Money {
        self.consumers.pnl
            + self.renewables.pnl
            + self.gas.pnl
            + self.nuclear.pnl
            + self.battery_discharge.pnl
            + self.battery_charge.pnl
            + self.market_bought.pnl
            + self.market_sold.pnl
            + self.imbalance.pnl
    }
}

pub fn compute_players_scores(
    trades: &[Trade],
    stacks_results: &HashMap<PlayerId, StackDispatchResults>,
) -> HashMap<PlayerId, PlayerScore> {
    stacks_results
        .iter()
        .map(|(player_id, results)| {
            (
                player_id.clone(),
                compute_player_score(player_id, results.plants_outputs(), trades),
            )
        })
        .collect()
}

fn compute_player_score(
    player_id: &PlayerId,
    outputs: &HashMap<PlantId, PlantOutput>,
    trades: &[Trade],
) -> PlayerScore {
    let market_position = trades
        .iter()
        .flat_map(|trade| trade.for_player(player_id))
        .fold(PlayerScore::default(), |acc, trade| acc + trade);
    let plant_position = outputs
        .values()
        .fold(PlayerScore::default(), |acc, output| acc + output);

    let mut player_position = plant_position + market_position;
    player_position.imbalance_cost = match player_position.balance {
        balance if balance > NO_POWER => {
            balance * TIMESTEP * EnergyCost::from(POSITIVE_IMBALANCE_COST)
        }
        balance if balance < NO_POWER => {
            balance * TIMESTEP * EnergyCost::from(NEGATIVE_IMBALANCE_COST)
        }
        _ => 0.into(),
    };
    player_position
}

pub fn compute_players_detailed_scores(
    trades: &[Trade],
    stacks_results: &HashMap<PlayerId, StackDispatchResults>,
) -> HashMap<PlayerId, PlayerDetailedScore> {
    stacks_results
        .iter()
        .map(|(player_id, results)| {
            (
                player_id.clone(),
                compute_player_detailed_score(player_id, results, trades),
            )
        })
        .collect()
}

fn compute_player_detailed_score(
    player_id: &PlayerId,
    outputs: &StackDispatchResults,
    trades: &[Trade],
) -> PlayerDetailedScore {
    let market_scores = process_player_trades(player_id, trades);
    let imbalance_score =
        compute_imbalance_score(market_scores.position() + outputs.aggregated_state().position());

    PlayerDetailedScore {
        consumers: outputs.aggregated_state().consumers().into(),
        renewables: outputs.aggregated_state().renewables().into(),
        gas: outputs.aggregated_state().gas().into(),
        nuclear: outputs.aggregated_state().nuclear().into(),
        battery_discharge: outputs.aggregated_state().battery_discharge().into(),
        battery_charge: outputs.aggregated_state().battery_charge().into(),
        market_bought: market_scores.bought,
        market_sold: market_scores.sold,
        imbalance: imbalance_score,
    }
}

struct MarketScore {
    bought: ScoreDetails,
    sold: ScoreDetails,
}

impl MarketScore {
    fn position(&self) -> Energy {
        self.bought.volume + self.sold.volume
    }
}

fn process_player_trades(player_id: &PlayerId, trades: &[Trade]) -> MarketScore {
    let mut volume_sold = Energy::from(0);
    let mut volume_bought = Energy::from(0);
    let mut pnl_sold = Money::from(0);
    let mut pnl_bought = Money::from(0);

    for trade in trades {
        for leg in trade.for_player(player_id) {
            match leg.direction {
                Direction::Buy => {
                    volume_bought = volume_bought + leg.volume;
                    // BUY -> our pnl decreases
                    pnl_bought = pnl_bought - leg.price * leg.volume;
                }
                Direction::Sell => {
                    // SELL -> our position decreases
                    volume_sold -= leg.volume;
                    pnl_sold = pnl_sold + leg.price * leg.volume;
                }
            }
        }
    }

    MarketScore {
        bought: ScoreDetails {
            volume: volume_bought,
            pnl: pnl_bought,
        },
        sold: ScoreDetails {
            volume: volume_sold,
            pnl: pnl_sold,
        },
    }
}

fn compute_imbalance_score(position: Energy) -> ScoreDetails {
    let imbalance_cost = match position {
        balance if balance > ZERO_ENERGY => balance * EnergyCost::from(POSITIVE_IMBALANCE_COST),
        balance if balance < ZERO_ENERGY => balance * EnergyCost::from(NEGATIVE_IMBALANCE_COST),
        _ => 0.into(),
    };
    ScoreDetails {
        volume: position,
        pnl: imbalance_cost,
    }
}

#[derive(Debug, PartialEq, Clone, Serialize)]
pub struct PlayerResult {
    pub player: PlayerId,
    pub rank: usize,
    pub score: Money,
}

pub fn compute_game_rankings(
    players_scores: &HashMap<PlayerId, HashMap<DeliveryPeriodId, PlayerScore>>,
) -> Vec<PlayerResult> {
    let mut scores: Vec<(PlayerId, Money)> = players_scores
        .iter()
        .map(|(player, score)| {
            (
                player.clone(),
                score
                    .iter()
                    .fold(Money::from(0), |acc, (_, s)| acc + s.pnl + s.imbalance_cost),
            )
        })
        .collect();
    scores.sort_by(|(_, a), (_, b)| b.cmp(a));
    scores
        .iter()
        .enumerate()
        .map(|(idx, (player_id, score))| PlayerResult {
            player: player_id.clone(),
            rank: idx + 1,
            score: *score,
        })
        .collect()
}

#[cfg(test)]
mod test_player_score_details {
    use crate::{
        game::scores::{PlayerDetailedScore, ScoreDetails},
        utils::units::{Energy, Money},
    };

    fn make_score(volume: isize, pnl: isize) -> ScoreDetails {
        ScoreDetails {
            volume: Energy::from(volume),
            pnl: Money::from(pnl),
        }
    }

    fn make_details(volumes: [isize; 9], pnls: [isize; 9]) -> PlayerDetailedScore {
        PlayerDetailedScore {
            consumers: make_score(volumes[0], pnls[0]),
            renewables: make_score(volumes[1], pnls[1]),
            gas: make_score(volumes[2], pnls[2]),
            nuclear: make_score(volumes[3], pnls[3]),
            battery_discharge: make_score(volumes[4], pnls[4]),
            battery_charge: make_score(volumes[5], pnls[5]),
            market_bought: make_score(volumes[6], pnls[6]),
            market_sold: make_score(volumes[7], pnls[7]),
            imbalance: make_score(volumes[8], pnls[8]),
        }
    }

    #[test]
    fn test_position_all_zeros() {
        let details = make_details([0; 9], [0; 9]);
        assert_eq!(details.position(), Energy::from(0));
    }

    #[test]
    fn test_position_sums_all_volumes() {
        let details = make_details([-100, 50, 30, 80, 20, -10, 40, -30, 5], [0; 9]);
        assert_eq!(
            details.position(),
            Energy::from(-100 + 50 + 30 + 80 + 20 - 10 + 40 - 30 + 5)
        );
    }

    #[test]
    fn test_pnl_all_zeros() {
        let details = make_details([0; 9], [0; 9]);
        assert_eq!(details.pnl(), Money::from(0));
    }

    #[test]
    fn test_pnl_sums_all_pnls() {
        let details = make_details([0; 9], [-500, 200, -300, 400, 100, -50, 800, -600, -150]);
        assert_eq!(
            details.pnl(),
            Money::from(-500 + 200 - 300 + 400 + 100 - 50 + 800 - 600 - 150)
        );
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::Utc;

    use crate::{
        game::scores::{
            NEGATIVE_IMBALANCE_COST, POSITIVE_IMBALANCE_COST, PlayerScore, compute_players_scores,
        },
        market::order_book::Trade,
        plants::{Output, PlantId, PlantOutput, StackAggregatedState, StackDispatchResults},
        player::PlayerId,
        utils::units::{Energy, EnergyCost, Money, Power},
    };

    #[test]
    fn test_scores_no_players() {
        assert_eq!(
            compute_players_scores(&Vec::new(), &HashMap::new()),
            HashMap::new()
        );
    }

    #[test]
    fn test_scores_no_trades_single_player_imbalanced() {
        let trades = Vec::new();
        let plants_outputs = HashMap::from([(
            PlayerId::from("player_1"),
            StackDispatchResults::new(
                HashMap::from([
                    (
                        PlantId::from("plant_1"),
                        PlantOutput {
                            setpoint: Power::from(100),
                            cost: Money::from(-100),
                        },
                    ),
                    (
                        PlantId::from("plant_2"),
                        PlantOutput {
                            setpoint: Power::from(200),
                            cost: Money::from(-500),
                        },
                    ),
                ]),
                StackAggregatedState::empty(),
            ),
        )]);

        assert_eq!(
            compute_players_scores(&trades, &plants_outputs),
            HashMap::from([(
                PlayerId::from("player_1"),
                PlayerScore {
                    balance: Power::from(300),
                    pnl: Money::from(-600),
                    imbalance_cost: Money::from(300 * POSITIVE_IMBALANCE_COST)
                }
            )])
        )
    }

    #[test]
    fn test_scores_with_trades_single_player_single_plant_imbalanced() {
        let trades = Vec::from([Trade {
            buyer: PlayerId::from("player_1"),
            seller: PlayerId::from("another_player"),
            execution_time: Utc::now(),
            price: EnergyCost::from(80),
            volume: Energy::from(100),
        }]);
        let plants_outputs = HashMap::from([(
            PlayerId::from("player_1"),
            StackDispatchResults::new(
                HashMap::from([
                    (
                        PlantId::from("plant_1"),
                        PlantOutput {
                            setpoint: Power::from(100),
                            cost: Money::from(-100),
                        },
                    ),
                    (
                        PlantId::from("plant_2"),
                        PlantOutput {
                            setpoint: Power::from(200),
                            cost: Money::from(-500),
                        },
                    ),
                ]),
                StackAggregatedState::empty(),
            ),
        )]);

        assert_eq!(
            compute_players_scores(&trades, &plants_outputs),
            HashMap::from([(
                PlayerId::from("player_1"),
                PlayerScore {
                    balance: Power::from(300 + 100),
                    pnl: Money::from(-600 - (80 * 100)),
                    imbalance_cost: Money::from(400 * POSITIVE_IMBALANCE_COST)
                }
            )])
        )
    }

    #[test]
    fn test_scores_multiple_players() {
        let trades = Vec::from([Trade {
            buyer: PlayerId::from("player_1"),
            seller: PlayerId::from("another_player"),
            execution_time: Utc::now(),
            price: EnergyCost::from(80),
            volume: Energy::from(100),
        }]);
        let plants_outputs = HashMap::from([
            (
                PlayerId::from("player_1"),
                StackDispatchResults::new(
                    HashMap::from([
                        (
                            PlantId::from("plant_1"),
                            PlantOutput {
                                setpoint: Power::from(100),
                                cost: Money::from(-100),
                            },
                        ),
                        (
                            PlantId::from("plant_2"),
                            PlantOutput {
                                setpoint: Power::from(200),
                                cost: Money::from(-500),
                            },
                        ),
                    ]),
                    StackAggregatedState::empty(),
                ),
            ),
            (
                PlayerId::from("another_player"),
                StackDispatchResults::new(
                    HashMap::from([(
                        PlantId::from("another_plant"),
                        PlantOutput {
                            setpoint: Power::from(-1000),
                            cost: Money::from(0),
                        },
                    )]),
                    StackAggregatedState::empty(),
                ),
            ),
        ]);

        assert_eq!(
            compute_players_scores(&trades, &plants_outputs),
            HashMap::from([
                (
                    PlayerId::from("player_1"),
                    PlayerScore {
                        balance: Power::from(300 + 100),
                        pnl: Money::from(-600 - (80 * 100)),
                        imbalance_cost: Money::from(400 * POSITIVE_IMBALANCE_COST)
                    }
                ),
                (
                    PlayerId::from("another_player"),
                    PlayerScore {
                        balance: Power::from(-1000 - 100),
                        #[allow(clippy::identity_op)] // Make test more explicit
                        pnl: Money::from(0 + (80 * 100)),
                        imbalance_cost: Money::from(-1100 * NEGATIVE_IMBALANCE_COST)
                    }
                )
            ])
        )
    }

    #[test]
    fn test_process_player_trades_no_trades_returns_zeros() {
        let player = PlayerId::from("player");
        let scores = super::process_player_trades(&player, &[]);
        assert_eq!(scores.bought.volume, Energy::from(0));
        assert_eq!(scores.bought.pnl, Money::from(0));
        assert_eq!(scores.sold.volume, Energy::from(0));
        assert_eq!(scores.sold.pnl, Money::from(0));
    }

    #[test]
    fn test_process_player_trades_single_buy() {
        let player = PlayerId::from("player");
        let trades = vec![Trade {
            buyer: player.clone(),
            seller: PlayerId::from("other"),
            price: EnergyCost::from(80),
            volume: Energy::from(100),
            execution_time: Utc::now(),
        }];
        let scores = super::process_player_trades(&player, &trades);
        assert_eq!(scores.bought.volume, Energy::from(100));
        assert_eq!(scores.bought.pnl, Money::from(-80 * 100)); // BUY -> negative money
        assert_eq!(scores.sold.volume, Energy::from(0));
        assert_eq!(scores.sold.pnl, Money::from(0));
    }

    #[test]
    fn test_process_player_trades_single_sell() {
        let player = PlayerId::from("player");
        let trades = vec![Trade {
            buyer: PlayerId::from("other"),
            seller: player.clone(),
            price: EnergyCost::from(90),
            volume: Energy::from(50),
            execution_time: Utc::now(),
        }];
        let scores = super::process_player_trades(&player, &trades);
        assert_eq!(scores.sold.volume, Energy::from(-50)); // BUY -> negative energy
        assert_eq!(scores.sold.pnl, Money::from(90 * 50));
        assert_eq!(scores.bought.volume, Energy::from(0));
        assert_eq!(scores.bought.pnl, Money::from(0));
    }

    #[test]
    fn test_process_player_trades_multiple_buys_are_summed() {
        let player = PlayerId::from("player");
        let trades = vec![
            Trade {
                buyer: player.clone(),
                seller: PlayerId::from("other"),
                price: EnergyCost::from(80),
                volume: Energy::from(100),
                execution_time: Utc::now(),
            },
            Trade {
                buyer: player.clone(),
                seller: PlayerId::from("other"),
                price: EnergyCost::from(70),
                volume: Energy::from(200),
                execution_time: Utc::now(),
            },
        ];
        let scores = super::process_player_trades(&player, &trades);
        assert_eq!(scores.bought.volume, Energy::from(300));
        assert_eq!(scores.bought.pnl, Money::from(-(80 * 100 + 70 * 200))); // BUY -> negative money
        assert_eq!(scores.sold.volume, Energy::from(0));
    }

    #[test]
    fn test_process_player_trades_ignores_other_players_trades() {
        let player = PlayerId::from("player");
        let trades = vec![Trade {
            buyer: PlayerId::from("other_a"),
            seller: PlayerId::from("other_b"),
            price: EnergyCost::from(80),
            volume: Energy::from(100),
            execution_time: Utc::now(),
        }];
        let scores = super::process_player_trades(&player, &trades);
        assert_eq!(scores.bought.volume, Energy::from(0));
        assert_eq!(scores.sold.volume, Energy::from(0));
    }

    #[test]
    fn test_process_player_trades_buys_and_sells_tracked_separately() {
        let player = PlayerId::from("player");
        let trades = vec![
            Trade {
                buyer: player.clone(),
                seller: PlayerId::from("other"),
                price: EnergyCost::from(80),
                volume: Energy::from(100),
                execution_time: Utc::now(),
            },
            Trade {
                buyer: PlayerId::from("other"),
                seller: player.clone(),
                price: EnergyCost::from(90),
                volume: Energy::from(60),
                execution_time: Utc::now(),
            },
        ];
        let scores = super::process_player_trades(&player, &trades);
        assert_eq!(scores.bought.volume, Energy::from(100));
        assert_eq!(scores.bought.pnl, Money::from(-80 * 100)); // BUY -> negative money
        assert_eq!(scores.sold.volume, Energy::from(-60)); // SELL -> negative volume
        assert_eq!(scores.sold.pnl, Money::from(90 * 60));
    }

    #[test]
    fn test_compute_imbalance_zero_position() {
        let result = super::compute_imbalance_score(Energy::from(0));
        assert_eq!(result.volume, Energy::from(0));
        assert_eq!(result.pnl, Money::from(0));
    }

    #[test]
    fn test_compute_imbalance_positive_position_uses_positive_cost() {
        let result = super::compute_imbalance_score(Energy::from(100));
        assert_eq!(result.volume, Energy::from(100));
        assert_eq!(result.pnl, Money::from(100 * POSITIVE_IMBALANCE_COST));
    }

    #[test]
    fn test_compute_imbalance_negative_position_uses_negative_cost() {
        let result = super::compute_imbalance_score(Energy::from(-200));
        assert_eq!(result.volume, Energy::from(-200));
        assert_eq!(result.pnl, Money::from(-200 * NEGATIVE_IMBALANCE_COST));
    }

    #[test]
    fn test_compute_player_score_details_gas_only_no_trades() {
        let player = PlayerId::from("player");
        let stack = StackDispatchResults::new(
            HashMap::new(),
            StackAggregatedState::new(
                Output::empty(),
                Output::empty(),
                Output::new(Energy::from(100), Money::from(500)),
                Output::empty(),
                Output::empty(),
                Output::empty(),
            ),
        );

        let result = super::compute_player_detailed_score(&player, &stack, &[]);

        assert_eq!(result.gas.volume, Energy::from(100));
        assert_eq!(result.gas.pnl, Money::from(500));
        assert_eq!(result.consumers.volume, Energy::from(0));
        assert_eq!(result.market_bought.volume, Energy::from(0));
        assert_eq!(result.market_sold.volume, Energy::from(0));
        assert_eq!(result.imbalance.volume, Energy::from(100));
        assert_eq!(
            result.imbalance.pnl,
            Money::from(100 * POSITIVE_IMBALANCE_COST)
        );
    }
}

#[cfg(test)]
mod test_pnl_ranking {
    use std::collections::HashMap;

    use crate::{
        game::{
            delivery_period::DeliveryPeriodId,
            scores::{PlayerResult, PlayerScore, compute_game_rankings},
        },
        player::PlayerId,
        utils::units::{Money, Power},
    };

    #[test]
    fn test_pnl_ranking() {
        let scores = HashMap::from([
            (
                PlayerId::from("toto"),
                HashMap::from([
                    (
                        DeliveryPeriodId::from(1),
                        PlayerScore {
                            balance: Power::from(0),
                            imbalance_cost: Money::from(0),
                            pnl: Money::from(1000),
                        },
                    ),
                    (
                        DeliveryPeriodId::from(2),
                        PlayerScore {
                            balance: Power::from(10),
                            imbalance_cost: Money::from(-60),
                            pnl: Money::from(1050),
                        },
                    ),
                ]),
            ),
            (
                PlayerId::from("other_player"),
                HashMap::from([
                    (
                        DeliveryPeriodId::from(1),
                        PlayerScore {
                            balance: Power::from(0),
                            imbalance_cost: Money::from(0),
                            pnl: Money::from(0),
                        },
                    ),
                    (
                        DeliveryPeriodId::from(2),
                        PlayerScore {
                            balance: Power::from(10),
                            imbalance_cost: Money::from(-60),
                            pnl: Money::from(0),
                        },
                    ),
                ]),
            ),
        ]);

        let rankings = compute_game_rankings(&scores);
        assert_eq!(
            rankings,
            vec![
                PlayerResult {
                    player: PlayerId::from("toto"),
                    rank: 1,
                    score: Money::from(1990),
                },
                PlayerResult {
                    player: PlayerId::from("other_player"),
                    rank: 2,
                    score: Money::from(-60),
                }
            ]
        )
    }
}
