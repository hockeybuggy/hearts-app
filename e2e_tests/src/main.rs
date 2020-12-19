use std::env;

use dotenv::dotenv;
use futures_util::{SinkExt, StreamExt};
use log::{debug, info};
use serde_json::{json, Value};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, WebSocketStream};

struct Player<T> {
    name: String,
    ws_stream: WebSocketStream<T>,
}

async fn send_message<T>(
    player: &mut Player<T>,
    message: Value,
) -> Result<(), tokio_tungstenite::tungstenite::Error>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    player
        .ws_stream
        .send(Message::Text(message.to_string()))
        .await?;
    debug!("{} Sent: {}", &player.name, message);
    return Ok(());
}

async fn receive_message<T>(
    player: &mut Player<T>
) -> Result<String, tokio_tungstenite::tungstenite::Error>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    let message = player.ws_stream.next().await.unwrap().unwrap();
    let message_text = &message.into_text().unwrap();
    debug!("{} Received: {}", player.name, message_text);
    return Ok(message_text.to_string());
}

struct Players<T> {
    host: Player<T>,
    amigo: Player<T>,
}

mod messages {
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Player {
        pub name: String,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Lobby {
        pub id: String,
        pub players: Vec<Player>,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct CreateLobbyReponse {
        pub lobby: Lobby,
    }

    #[derive(Serialize, Deserialize, Debug)]
    pub struct JoinLobbyReponse {
        pub lobby: Lobby,
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    let websocket_url = env::var("WEBSOCKET_URL").expect("Could not get WEBSOCKET_URL env var");
    let url = url::Url::parse(&websocket_url).expect("WEBSOCKET_URL not a valid url");

    let host_player_name = "Host".to_owned();
    let amigo_player_name = "Amgio".to_owned();
    info!("Opening websocket connections for the simulated players.");
    let (host_ws_stream, _) = connect_async(url.clone())
        .await
        .expect(&format!("{} failed to connect", &host_player_name).to_owned());
    let (amigo_ws_stream, _) = connect_async(url.clone())
        .await
        .expect(&format!("{} failed to connect", &amigo_player_name).to_owned());
    let mut players = Players {
        host: Player {
            name: host_player_name,
            ws_stream: host_ws_stream,
        },
        amigo: Player {
            name: amigo_player_name,
            ws_stream: amigo_ws_stream,
        },
    };

    info!("{} creates a lobby.", &players.host.name);
    let create_lobby_message = json!({
      "action": "hearts",
      "type": "lobby_action_create",
      "name": json!(players.host.name),
    });

    send_message(&mut players.host, create_lobby_message)
        .await
        .expect("Failed to send create_lobby_message");

    let message = receive_message(&mut players.host)
        .await
        .expect("Failed to receive response to create_lobby_message");

    // Get `lobby.id` from the message.
    let create_lobby_response: messages::CreateLobbyReponse =
        serde_json::from_str(&message).unwrap();
    let create_lobby_player_names: String = create_lobby_response
        .lobby
        .players
        .iter()
        .map(|p| p.name.clone())
        .collect::<Vec<String>>()
        .join(", ");
    info!(
        "{} recieves a update message with the lobby code {}.",
        &players.host.name, &create_lobby_response.lobby.id,
    );
    info!("The lobby has the players: {}", create_lobby_player_names);

    let join_lobby_message = json!({
      "action": "hearts",
      "type": "lobby_action_join",
      "name": json!(players.amigo.name),
      // TODO this should be the `lobby_code so that people don't have to type out a uuid
      "lobby_code": json!(create_lobby_response.lobby.id),
    });

    info!(
        "{} sends the lobby code {} to their friend Amigo.",
        &players.host.name, create_lobby_response.lobby.id
    );
    send_message(&mut players.amigo, join_lobby_message)
        .await
        .expect("Failed to send join_lobby_message");

    let message = receive_message(&mut players.host)
        .await
        .expect(&format!("{} failed to receive update message", &players.host.name).to_owned());
    let join_lobby_response: messages::JoinLobbyReponse = serde_json::from_str(&message).unwrap();
    debug!("{:?}", join_lobby_response);

    let join_lobby_player_names: String = join_lobby_response
        .lobby
        .players
        .iter()
        .map(|p| p.name.clone())
        .collect::<Vec<String>>()
        .join(", ");
    info!(
        "{} receives a update message containing the players: {}",
        &players.host.name, join_lobby_player_names
    );

    let message = receive_message(&mut players.amigo)
        .await
        .expect(&format!("{} failed to receive update message", &players.host.name).to_owned());
    let join_lobby_response: messages::JoinLobbyReponse = serde_json::from_str(&message).unwrap();
    debug!("{:?}", join_lobby_response);

    let join_lobby_player_names: String = join_lobby_response
        .lobby
        .players
        .iter()
        .map(|p| p.name.clone())
        .collect::<Vec<String>>()
        .join(", ");
    info!(
        "{} receives a update message containing the players: {}",
        &players.amigo.name, join_lobby_player_names
    );

    players.host.ws_stream.close(None).await.unwrap();
    players.amigo.ws_stream.close(None).await.unwrap();

    info!("Websockets closed: 👋");
}
