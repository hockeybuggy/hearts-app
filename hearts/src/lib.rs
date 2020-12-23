use chrono::Utc;
use dynomite::dynamodb::DynamoDbClient;
use lambda::Context;
use serde::Deserialize;
use serde_json::{json, Value};

use common::lobby;
use common::websocket_client::WebSocketClient;
use messages::Message;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    request_context: RequestContext,
    body: String, // parse this into json
}

impl Event {
    fn message(&self) -> Option<messages::Message> {
        serde_json::from_str::<messages::Message>(&self.body).ok()
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
    let now = Utc::now();
    let endpoint = endpoint(&event.request_context);
    let connection_id = event.request_context.connection_id;

    match message {
        Some(Message::LobbyActionCreate(e)) => {
            let lobby =
                lobby::LobbyService::create(&ddb_client, &now, &e.name, &connection_id).await?;
            for player in lobby.players.iter() {
                let ws_client = WebSocketClient::new(&endpoint);
                ws_client
                    .post_to_connection(
                        &player.connection_id,
                        Message::LobbyActionCreateResponse(messages::LobbyActionCreateResponse {
                            lobby: messages::Lobby {
                                id: lobby.id.clone(),
                                players: lobby
                                    .players
                                    .iter()
                                    .map(|p| messages::Player {
                                        name: p.name.clone(),
                                        connection_id: p.connection_id.clone(),
                                    })
                                    .collect(),
                            },
                        }),
                    )
                    .await?;
            }
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
            for player in lobby.players.iter() {
                let ws_client = WebSocketClient::new(&endpoint);
                ws_client
                    .post_to_connection(
                        &player.connection_id,
                        Message::LobbyActionJoinResponse(messages::LobbyActionJoinResponse {
                            // TODO this seems like something to extract
                            lobby: messages::Lobby {
                                id: lobby.id.clone(),
                                players: lobby
                                    .players
                                    .iter()
                                    .map(|p| messages::Player {
                                        name: p.name.clone(),
                                        connection_id: p.connection_id.clone(),
                                    })
                                    .collect(),
                            },
                        }),
                    )
                    .await?;
            }
        }
        _ => {
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
