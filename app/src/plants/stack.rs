use std::collections::HashMap;

use serde::Deserialize;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use uuid::Uuid;

use crate::{market::PlayerConnection, player::PlayerMessage};

use super::{battery::Battery, gas_plant::GasPlant, PowerPlant, PowerPlantPublicRepr};

#[derive(Debug, Deserialize)]
pub struct ProgramPlant {
    pub plant_id: String,
    pub setpoint: isize,
}

pub enum StackMessage {
    ProgramSetpoint(ProgramPlant),
    RegisterPlayerConnection(PlayerConnection),
}

/// A stack is the collection of power plants owned by a given player
pub struct StackActor {
    player_id: String,
    plants: HashMap<String, Box<dyn PowerPlant + Send + Sync>>,
    tx: Sender<StackMessage>,
    rx: Receiver<StackMessage>,
    player_connections: Vec<PlayerConnection>,
}

impl StackActor {
    pub fn new(player_id: String) -> StackActor {
        let (tx, rx) = channel::<StackMessage>(16);

        StackActor {
            player_id,
            plants: default_plants(),
            player_connections: Vec::new(),
            tx,
            rx,
        }
    }

    pub fn get_tx(&self) -> Sender<StackMessage> {
        self.tx.clone()
    }

    pub async fn start(&mut self) {
        while let Some(message) = self.rx.recv().await {
            match message {
                StackMessage::RegisterPlayerConnection(connection) => {
                    self.handle_player_connection(connection).await;
                }
                StackMessage::ProgramSetpoint(ProgramPlant { plant_id, setpoint }) => {
                    let _ = self.program_plant_setpoint(plant_id, setpoint).await;
                }
            }
        }
    }

    async fn handle_player_connection(&mut self, conn: PlayerConnection) {
        if conn.player_id != self.player_id {
            return;
        }
        let conn_id = conn.id.clone();
        self.player_connections.push(conn);
        self.send_stack_snapshot(conn_id).await;
    }

    async fn send_stack_snapshot(&self, conn_id: String) {
        let Some(conn) = self.player_connections.iter().find(|c| c.id == conn_id) else {
            return;
        };

        let stack_snapshot: HashMap<String, PowerPlantPublicRepr> = self
            .plants
            .iter()
            .map(|(id, p)| (id.to_owned(), p.current_state()))
            .collect();

        let _ = conn
            .tx
            .send(PlayerMessage::StackSnapshot {
                plants: stack_snapshot,
            })
            .await;
    }

    async fn program_plant_setpoint(&mut self, plant_id: String, setpoint: isize) {
        if let Some(plant) = self.plants.get_mut(&plant_id) {
            let cost = plant.program_setpoint(setpoint);
            println!("Programmed setpoint {setpoint} for plant {plant_id} (cost: {cost}");
        };
    }
}

fn default_plants() -> HashMap<String, Box<dyn PowerPlant + Send + Sync>> {
    let mut map: HashMap<String, Box<dyn PowerPlant + Send + Sync>> = HashMap::new();
    map.insert(
        Uuid::new_v4().to_string(),
        Box::new(Battery::new(1_000, 500)),
    );
    map.insert(
        Uuid::new_v4().to_string(),
        Box::new(GasPlant::new(85, 1000)),
    );
    map
}
