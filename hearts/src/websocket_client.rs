use rusoto_apigatewaymanagementapi::{
    ApiGatewayManagementApi, ApiGatewayManagementApiClient, PostToConnectionError,
    PostToConnectionRequest,
};
use rusoto_core::Region;
use serde::Serialize;
use serde_json::json;

pub struct WebSocketClient {
    client: ApiGatewayManagementApiClient,
}

impl WebSocketClient {
    pub fn new(endpoint: &String) -> Self {
        let client = ApiGatewayManagementApiClient::new(Region::Custom {
            name: Region::CaCentral1.name().into(),
            endpoint: endpoint.to_string(),
        });
        return Self { client };
    }

    pub async fn post_to_connection<T>(
        self,
        connection_id: &String,
        message: T,
    ) -> Result<(), rusoto_core::RusotoError<PostToConnectionError>>
    where
        T: Serialize,
    {
        self.client
            .post_to_connection(PostToConnectionRequest {
                connection_id: connection_id.clone(),
                data: serde_json::to_vec(&json!(message))
                    .unwrap_or_default()
                    .into(),
            })
            .await
        // TODO elsewhere we have used `PostToConnectionError::Gone(_)` to delete connection
        // objects. We should likely be doing something like that.
    }
}
