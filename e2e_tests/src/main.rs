use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, WebSocketStream};

struct Player<T> {
    name: String,
    ws_stream: WebSocketStream<T>,
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

    let connect_str = r#"
    {
      "action": "hearts",
      "type": "lobby_action_create",
      "name": "Host"
    }"#;
    players
        .host
        .ws_stream
        .send(Message::Text(connect_str.to_owned()))
        .await
        .expect("Failed to send message");

    let message = players.host.ws_stream.next().await.unwrap().unwrap();
    println!("{} Received: {}", &players.host.name, message);

    // Get `lobby.id` from the message.
    let x = &message.into_text().unwrap();
    let lobby: CreateLobbyReponse = serde_json::from_str(x).unwrap();
    // Send join
    println!("{} lobby: {:?}", &players.host.name, &lobby);

    players.host.ws_stream.close(None).await.unwrap();
    players.amigo.ws_stream.close(None).await.unwrap();
}
