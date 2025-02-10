use std::collections::HashMap;

use serde::Deserialize;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use uuid::Uuid;

use crate::market::PlayerConnection;

use super::{battery::Battery, gas_plant::GasPlant, PowerPlant};

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
                    self.handle_player_connection(connection);
                }
                StackMessage::ProgramSetpoint(ProgramPlant { plant_id, setpoint }) => {
                    let _ = self.program_plant_setpoint(plant_id, setpoint).await;
                }
            }
        }
    }

    fn handle_player_connection(&mut self, conn: PlayerConnection) {
        if conn.player_id != self.player_id {
            return;
        }

        self.player_connections.push(conn);
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
    map.insert(Uuid::new_v4().to_string(), Box::new(GasPlant::new(85)));
    map
}
