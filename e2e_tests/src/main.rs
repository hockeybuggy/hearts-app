use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
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
    println!("{} Sent: {}", &player.name, message);
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
    println!("{} Received: {}", player.name, message_text);
    return Ok(message_text.to_string());
}

struct Players<T> {
    host: Player<T>,
    amigo: Player<T>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Lobby {
    id: String,
}
#[derive(Serialize, Deserialize, Debug)]
struct CreateLobbyReponse {
    lobby: Lobby,
}

#[tokio::main]
async fn main() {
    env_logger::init();

    // TODO get from env
    let connect_addr = "wss://rse5mmis8e.execute-api.ca-central-1.amazonaws.com/dev".to_owned();
    let url = url::Url::parse(&connect_addr).unwrap();

    let (host_ws_stream, _) = connect_async(url.clone()).await.expect("Failed to connect");
    let (amigo_ws_stream, _) = connect_async(url.clone()).await.expect("Failed to connect");
    let mut players = Players {
        host: Player {
            name: "Host".to_owned(),
            ws_stream: host_ws_stream,
        },
        amigo: Player {
            name: "Amigo".to_owned(),
            ws_stream: amigo_ws_stream,
        },
    };

    let create_lobby_message = json!({
      "action": "hearts",
      "type": "lobby_action_create",
      "name": "Host"
    });

    send_message(&mut players.host, create_lobby_message)
        .await
        .expect("Failed to send create_lobby_message");

    let message = receive_message(&mut players.host)
        .await
        .expect("Failed to receive respond to create_lobby_message");

    // Get `lobby.id` from the message.
    let create_lobby_response: CreateLobbyReponse = serde_json::from_str(&message).unwrap();

    // Send join
    println!("{} lobby: {:?}", &players.host.name, &create_lobby_response);

    let join_lobby_message = json!({
      "action": "hearts",
      "type": "lobby_action_join",
      "name": "Amigo",
      // TODO this should be the `lobby_code so that people don't have to type out a uuid
      "lobby_code": json!(create_lobby_response.lobby.id),
    });

    send_message(&mut players.amigo, join_lobby_message)
        .await
        .expect("Failed to send join_lobby_message");

    let message = receive_message(&mut players.amigo)
        .await
        .expect("Failed to receive respond to join_lobby_message");

    players.host.ws_stream.close(None).await.unwrap();
    players.amigo.ws_stream.close(None).await.unwrap();

    println!("Websockets closed: 👋");
}
