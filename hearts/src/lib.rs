use chrono::Utc;
use dynomite::dynamodb::DynamoDbClient;
use lambda::Context;
use serde::Deserialize;
use serde_json::{json, Value};

mod lobby;
mod websocket_client;

use websocket_client::WebSocketClient;

#[derive(Deserialize, Debug, PartialEq)]
struct LobbyActionCreate {
    name: String,
}

#[derive(Deserialize, Debug, PartialEq)]
struct LobbyActionJoin {
    name: String,
    lobby_code: String,
}

/// the structure of the client payload (action aside)
#[derive(Deserialize, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Message {
    LobbyActionCreate(LobbyActionCreate),
    LobbyActionJoin(LobbyActionJoin),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    request_context: RequestContext,
    body: String, // parse this into json
}

impl Event {
    fn message(&self) -> Option<Message> {
        serde_json::from_str::<Message>(&self.body).ok()
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RequestContext {
    connection_id: String,
    domain_name: String,
    stage: String,
}

fn endpoint(ctx: &RequestContext) -> String {
    format!("https://{}/{}", ctx.domain_name, ctx.stage)
}

async fn inner_deliver(
    event: Event,
    _context: Context,
) -> Result<(), Box<dyn std::error::Error + Sync + Send + 'static>> {
    log::info!("recv {}", event.body);
    let message = event.message();
    log::info!("message {:?}", message);

    let ddb_client = DynamoDbClient::new(Default::default());
    let ws_client = WebSocketClient::new(&endpoint(&event.request_context));
    let now = Utc::now();
    let connection_id = event.request_context.connection_id;

    match message {
        Some(Message::LobbyActionCreate(e)) => {
            let lobby =
                lobby::LobbyService::create(&ddb_client, &now, &e.name, &connection_id).await?;
            ws_client
                .post_to_connection(&connection_id, json!({"status": "success", "lobby": lobby}))
                .await?;
        }
        Some(Message::LobbyActionJoin(e)) => {
            let lobby = lobby::LobbyService::join(
                &ddb_client,
                &now,
                &e.lobby_code,
                &e.name,
                &connection_id,
            )
            .await?;
            ws_client
                .post_to_connection(&connection_id, json!({"status": "success", "lobby": lobby}))
                .await?;
        }
        None => {
            log::info!("Invalid action");
        }
    }
    Ok(())
}

pub async fn deliver(
    event: Event,
    context: Context,
) -> Result<Value, Box<dyn std::error::Error + Sync + Send + 'static>> {
    let inner_result = inner_deliver(event, context).await;
    match inner_result {
        Ok(_) => {
            return Ok(json!({ "statusCode": 200 }));
        }
        Err(e) => {
            log::error!("{:?}", e);
            return Err(e);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_create_lobby_event() {
        let event = serde_json::from_str::<Event>(include_str!("../tests/data/create_lobby.json"))
            .expect("failed to deserialize send event");
        assert_eq!(
            event.message().and_then(|m| Some(m)),
            Some(Message::LobbyActionCreate(LobbyActionCreate {
                name: "Host".to_string()
            }))
        )
    }

    #[test]
    fn deserialize_join_lobby_event() {
        let event = serde_json::from_str::<Event>(include_str!("../tests/data/join_lobby.json"))
            .expect("failed to deserialize send event");
        assert_eq!(
            event.message().and_then(|m| Some(m)),
            Some(Message::LobbyActionJoin(LobbyActionJoin {
                name: "Host".to_string(),
                lobby_code: "hljk".to_string(),
            }))
        )
    }
}
