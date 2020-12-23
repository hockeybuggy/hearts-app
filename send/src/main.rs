use dynomite::dynamodb::DynamoDbClient;
use lambda::{handler_fn, Context};
use serde::Deserialize;
use serde_json::{json, Value};

use common::lobby;
use common::websocket_client::WebSocketClient;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Event {
    request_context: RequestContext,
    body: String, // parse this into json
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RequestContext {
    connection_id: String,
    domain_name: String,
    stage: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Sync + Send + 'static>> {
    env_logger::init();
    lambda::run(handler_fn(deliver)).await?;
    Ok(())
}

fn endpoint(ctx: &RequestContext) -> String {
    format!("https://{}/{}", ctx.domain_name, ctx.stage)
}

async fn deliver(
    event: Event,
    _context: Context,
) -> Result<Value, Box<dyn std::error::Error + Sync + Send + 'static>> {
    log::info!("recv {}", event.body);

    let ddb_client = DynamoDbClient::new(Default::default());
    let endpoint = endpoint(&event.request_context);
    let connection_id = event.request_context.connection_id;

    let message = serde_json::from_str::<messages::LobbyMessage>(&event.body)
        .ok()
        .expect("Invalid message");
    let lobby_code = message.lobby_code;
    let body = message.body;

    let lobby = lobby::LobbyService::get(&ddb_client, &lobby_code).await?;

    // Send the message body to all users who are not the current user
    let sender = lobby
        .players
        .iter()
        .find(|p| p.connection_id == connection_id);

    for player in lobby
        .players
        .iter()
        .filter(|p| p.connection_id != connection_id)
    {
        let ws_client = WebSocketClient::new(&endpoint);
        ws_client
            .post_to_connection(
                &player.connection_id,
                messages::Message::LobbyMessageResponse(messages::LobbyMessageResponse {
                    // TODO don't unwrap here.
                    name: sender.unwrap().name.clone(),
                    body: body.clone(),
                }),
            )
            .await?;
    }

    Ok(json!({
        "statusCode": 200
    }))
}

#[cfg(test)]
mod tests {
    // use super::*;

    // #[test]
    // fn deserialize_send_event_with_message() {
    //     let event =
    //         serde_json::from_str::<Event>(include_str!("../tests/data/send-something.json"))
    //             .expect("failed to deserialize send event");
    //     assert_eq!(event.message().and_then(|m| Some(m)), Some("howdy".into()))
    // }

    // #[test]
    // fn deserialize_send_event_without_message() {
    //     let event = serde_json::from_str::<Event>(include_str!("../tests/data/send-nothing.json"))
    //         .expect("failed to deserialize send event");
    //     assert_eq!(event.message(), None)
    // }

    // #[test]
    // fn formats_endpoint() {
    //     assert_eq!(
    //         endpoint(&RequestContext {
    //             domain_name: "xxx.execute-api.ca-central-1.amazonaws.com".into(),
    //             stage: "dev".into()
    //         }),
    //         "https://xxx.execute-api.ca-central-1.amazonaws.com/dev"
    //     )
    // }
}
