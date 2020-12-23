use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Player {
    pub name: String,
    pub connection_id: String,
}

pub type LobbyId = String;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Lobby {
    pub id: LobbyId,
    pub players: Vec<Player>,
}
