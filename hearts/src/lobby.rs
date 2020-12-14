use std::collections::HashMap;
use std::env;

use dynomite::{
    dynamodb::{DynamoDb, DynamoDbClient, GetItemInput, PutItemInput},
    AttributeValue, Attributes, FromAttributes, Item,
};
use serde::Serialize;
use uuid::Uuid;

pub struct LobbyService;

#[derive(Attributes, Debug, Serialize, Clone)]
pub struct Player {
    name: String,
    connection_id: String,
}

#[derive(Item, Debug, Serialize, Clone)]
pub struct Lobby {
    #[dynomite(partition_key)]
    id: Uuid,
    code: String,
    players: Vec<Player>,
}

impl LobbyService {
    pub async fn create(
        ddb: &DynamoDbClient,
        host_name: &String,
        connection_id: &String,
    ) -> Result<Lobby, Box<dyn std::error::Error + Sync + Send + 'static>> {
        log::info!("Create: {}", host_name);

        let mut players = Vec::new();
        players.push(Player {
            name: host_name.to_string(),
            connection_id: connection_id.to_string(),
        });
        let lobby_code = "1231".to_owned();
        let lobby = Lobby {
            id: Uuid::new_v4(),
            code: lobby_code,
            players,
        };

        log::info!("Lobby: {:?}", lobby);
        LobbyRepo::put(ddb, &lobby).await?;
        return Ok(lobby);
    }

    pub async fn join(
        ddb: &DynamoDbClient,
        lobby_code: &String,
        player_name: &String,
    ) -> Result<Lobby, Box<dyn std::error::Error + Sync + Send + 'static>> {
        log::info!("Join: {} {}", lobby_code, player_name);

        // TODO this is logically wrong at the moment since we're not using lobby codes, but the
        // lobby ids.
        let maybe_lobby = LobbyRepo::get(ddb, &Uuid::parse_str(lobby_code)?).await?;
        log::info!("LobbyService::join get result: {:?}", &maybe_lobby);

        // TODO unwrapping this is not good. It's pretty valid for the game there are trying to
        // join to not exist
        Ok(maybe_lobby.unwrap())
        // TODO Get from DB, add player, then update db (in race condition tollerant way)
    }
}

struct LobbyRepo;

impl LobbyRepo {
    pub async fn get(
        ddb: &DynamoDbClient,
        lobby_id: &Uuid,
    ) -> Result<Option<Lobby>, Box<dyn std::error::Error + Sync + Send + 'static>> {
        let table_name = env::var("tableName")?;
        let maybe_lobby = ddb
            .get_item(GetItemInput {
                table_name: table_name.clone(),
                key: {
                    let mut x = HashMap::new();
                    x.insert(
                        "id".to_owned(),
                        AttributeValue {
                            s: Some(lobby_id.to_string()),
                            ..AttributeValue::default()
                        },
                    );
                    x
                },
                ..GetItemInput::default()
            })
            .await?
            .item
            .map(|attrs| Lobby::from_attrs(attrs))
            .transpose()?;

        log::info!("LobbyRepo::get result: {:?}", &maybe_lobby);
        return Ok(maybe_lobby);
    }

    pub async fn put(
        ddb: &DynamoDbClient,
        lobby: &Lobby,
    ) -> Result<(), Box<dyn std::error::Error + Sync + Send + 'static>> {
        let item = lobby.clone().into();
        let table_name = env::var("tableName")?;
        let result = ddb
            .put_item(PutItemInput {
                table_name: table_name.clone(),
                item,
                ..PutItemInput::default()
            })
            .await?;
        log::info!("LobbyRepo::put result: {:?}", result);
        return Ok(());
    }
}
