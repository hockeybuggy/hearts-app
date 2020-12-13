use dynomite::{
    dynamodb::{DynamoDb, DynamoDbClient, PutItemInput},
    Attributes, Item,
};
use std::env;
use uuid::Uuid;

pub struct LobbyService;

#[derive(Attributes, Debug)]
struct Player {
    name: String,
    connection_id: String,
}

#[derive(Item, Debug)]
struct Lobby {
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
    ) -> Result<(), Box<dyn std::error::Error + Sync + Send + 'static>> {
        log::info!("Create: {}", host_name);
        let mut players = Vec::new();
        players.push(Player {
            name: host_name.to_string(),
            connection_id: connection_id.to_string(),
        });
        let lobby_code = "1231".to_owned();
        let item = Lobby {
            id: Uuid::new_v4(),
            code: lobby_code,
            players,
        }
        .into();
        log::info!("Lobby: {:?}", item);
        let table_name = env::var("tableName")?;
        let result = ddb
            .put_item(PutItemInput {
                table_name: table_name.clone(),
                // convert book into it's attribute map representation
                item,
                ..PutItemInput::default()
            })
            .await?;
        log::info!("Result: {:?}", result);
        return Ok(());
    }

    pub fn join(
        _ddb: &DynamoDbClient,
        lobby_code: &String,
        player_name: &String,
    ) {
        log::info!("Join: {} {}", lobby_code, player_name);
        // TODO Get from DB, add player, then update db (in race condition tollerant way)
    }
}
