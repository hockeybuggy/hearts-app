use dynomite::Item;
use uuid::Uuid;

pub struct LobbyService;

#[derive(Debug)]
struct Player {
    name: String,
    connection_id: String,
}

#[derive(Debug)]
struct Lobby {
    code: String,
    players: Vec<Player>,
}

impl LobbyService {
    pub fn create(
        host_name: String,
        connection_id: String,
    ) {
        log::info!("Create: {}", host_name);
        let mut players = Vec::new();
        players.push(Player {
            name: host_name,
            connection_id,
        });
        let lobby_code = "1231".to_owned();
        let item = Lobby {
            code: lobby_code,
            players,
        }
        .into();
        log::info!("Lobby: {:?}", item);
        // TODO push into db
        println!(
            "put_item() result {:#?}",
            client
                .put_item(PutItemInput {
                    table_name: table_name.clone(),
                    // convert book into it's attribute map representation
                    item: Book {
                        id: Uuid::new_v4(),
                        title: "rust and beyond".into(),
                        authors: Some(vec![Author {
                            id: Uuid::new_v4(),
                            name: "Jim Ferris".into(),
                        }]),
                    }
                    .into(),
                    ..PutItemInput::default()
                })
                .await?
        );
    }

    pub fn join(
        lobby_code: String,
        player_name: String,
    ) {
        log::info!("Join: {} {}", lobby_code, player_name);
        // TODO Get from DB, add player, then update db (in race condition tollerant way)
    }
}
