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
        let lobby = Lobby {
            code: lobby_code,
            players,
        };
        log::info!("Lobby: {:?}", lobby);
        // TODO push into db
    }

    pub fn join(
        lobby_code: String,
        player_name: String,
    ) {
        log::info!("Join: {} {}", lobby_code, player_name);
        // TODO Get from DB, add player, then update db (in race condition tollerant way)
    }
}
