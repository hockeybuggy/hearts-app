use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Player {
    pub name: String,
    pub connection_id: String,
}

pub type LobbyId = String;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Lobby {
    pub id: LobbyId,
    pub players: Vec<Player>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct LobbyActionCreate {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct LobbyActionCreateResponse {
    pub lobby: Lobby,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct LobbyActionJoin {
    pub name: String,
    pub lobby_code: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct LobbyActionJoinResponse {
    pub lobby: Lobby,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct LobbyMessage {
    pub lobby_code: String,
    pub body: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct LobbyMessageResponse {
    pub name: String,
    pub body: String,
}

/// the structure of the client payload (action aside)
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    LobbyActionCreate(LobbyActionCreate),
    LobbyActionCreateResponse(LobbyActionCreateResponse),
    LobbyActionJoin(LobbyActionJoin),
    LobbyActionJoinResponse(LobbyActionJoinResponse),
    LobbyMessage(LobbyMessage),
    LobbyMessageResponse(LobbyMessageResponse),
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn deserialize_create_lobby_event() {
//         let event = serde_json::from_str::<Event>(include_str!("../tests/data/create_lobby.json"))
//             .expect("failed to deserialize send event");
//         assert_eq!(
//             event.message().and_then(|m| Some(m)),
//             Some(Message::LobbyActionCreate(Message::LobbyActionCreate {
//                 name: "Host".to_string()
//             }))
//         )
//     }

//     #[test]
//     fn deserialize_join_lobby_event() {
//         let event = serde_json::from_str::<Event>(include_str!("../tests/data/join_lobby.json"))
//             .expect("failed to deserialize send event");
//         assert_eq!(
//             event.message().and_then(|m| Some(m)),
//             Some(Message::LobbyActionJoin(LobbyActionJoin {
//                 name: "Host".to_string(),
//                 lobby_code: "hljk".to_string(),
//             }))
//         )
//     }
// }
