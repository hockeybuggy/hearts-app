pub struct LobbyService;

impl LobbyService {
    pub fn create(host_name: String) {
        log::info!("Create: {}", host_name);
    }

    pub fn join(
        lobby_code: String,
        player_name: String,
    ) {
        log::info!("Join: {} {}", lobby_code, player_name);
    }
}
