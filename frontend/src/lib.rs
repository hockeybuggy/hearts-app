#![recursion_limit = "256"]

use wasm_bindgen::prelude::*;

use anyhow::Error;
use serde_derive::Deserialize;
use serde_json::{json, Value};
use yew::format::Json;
use yew::prelude::*;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};

struct Model {
    link: ComponentLink<Self>,
    lobby: Option<Lobby>,
    name: String,
    lobby_code_input: String,
    ws: Option<WebSocketTask>,
}

impl Model {
    fn view_lobby(&self) -> Html {
        if let Some(lobby) = &self.lobby {
            let player_names = lobby
                .players
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<String>>()
                .join(", ");

            html! {
                <div>
                    <p>{ "Lobby code:" }{ lobby.id.clone() }</p>
                    <p>{ "All players: "}{ player_names }</p>
                </div>
            }
        } else {
            let create_lobby_message = json!({
              "action": "hearts",
              "type": "lobby_action_create",
              "name": self.name.clone(),
            });
            let join_lobby_message = json!({
              "action": "hearts",
              "type": "lobby_action_join",
              "lobby_code": self.lobby_code_input.clone(),
              "name": self.name.clone(),
            });
            html! {
                <div>
                    <p>{ "Not in a lobby." }</p>

                    <div>
                        <button
                            class="w-32 m-4 disabled:opacity-50 bg-blue-200 hover:bg-blue-300 rounded-lg shadow-md"
                            disabled={self.ws.is_none() || self.name.is_empty()}
                            onclick=self.link.callback(move |_| {
                                Msg::WsAction(WsAction::SendData(create_lobby_message.clone()))
                            })
                        >
                            { "Create lobby" }
                        </button>
                    </div>

                    <div>
                        <input
                            value=&self.lobby_code_input
                            oninput=self.link.callback(|e: InputData| Msg::LobbyCodeInputChange(e.value))
                            placeholder="Lobby code"
                            class="m-4 focus:ring-2 focus:ring-blue-600 rounded-lg shadow-md"
                        />
                        <button
                            class="w-32 m-4 disabled:opacity-50 bg-blue-200 hover:bg-blue-300 rounded-lg shadow-md"
                            disabled={self.ws.is_none() || self.name.is_empty()}
                            onclick=self.link.callback(move |_| {
                                Msg::WsAction(WsAction::SendData(join_lobby_message.clone()))
                            })
                        >
                            { "Join lobby" }
                        </button>
                    </div>
                </div>
            }
        }
    }

    fn view_name(&self) -> Html {
        if self.lobby.is_some() {
            html! {
                <div>
                    { "Your name: " }
                    { self.name.clone() }
                </div>
            }
        } else {
            html! {
                <div>
                    <input
                        value=&self.name
                        oninput=self.link.callback(|e: InputData| Msg::NameInputChange(e.value))
                        placeholder="Pick a name your friends will see."
                        class="m-4 focus:ring-2 focus:ring-blue-600 rounded-lg shadow-md"
                    />
                </div>
            }
        }
    }

    fn view_connection_status(&self) -> Html {
        if self.ws.is_some() {
            html! {
                <div class="flex justify-end p-4 shadow-md">
                    <p>{ "✓" }</p>
                    <button
                        class="w-32 bg-red-200 hover:bg-red-300 rounded-lg shadow-md"
                        onclick=self.link.callback(|_| Msg::WsAction(WsAction::Disconnect))
                    >
                        { "Disconnect" }
                    </button>
                </div>
            }
        } else {
            html! {
                <div class="flex justify-end p-4 shadow-md">
                    <p>{ "𝙓" }</p>
                    <button
                        class="w-32 bg-blue-200 hover:bg-blue-300 rounded-lg shadow-md"
                        onclick=self.link.callback(|_| Msg::WsAction(WsAction::Connect))
                    >
                        { "Connect" }
                    </button>
                </div>
            }
        }
    }
}

enum WsAction {
    Connect,
    SendData(Value),
    Disconnect,
    Lost,
}

enum Msg {
    Ignore,
    WsAction(WsAction),
    WsReady(Result<WsResponse, Error>),
    NameInputChange(String),
    LobbyCodeInputChange(String),
}

// TODO the messages between the client and the server should be extracted into another crate.

#[derive(Deserialize, Debug)]
pub struct Player {
    pub name: String,
    pub connection_id: String,
}

pub type LobbyId = String;

#[derive(Deserialize, Debug)]
pub struct Lobby {
    id: LobbyId,
    pub players: Vec<Player>,
}

/// This type is an expected response from a websocket connection.
#[derive(Deserialize, Debug)]
pub struct WsResponse {
    lobby: Lobby,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(
        _: Self::Properties,
        link: ComponentLink<Self>,
    ) -> Self {
        Self {
            link,
            name: "".to_owned(),
            lobby: None,
            lobby_code_input: "".to_owned(),
            ws: None,
        }
    }

    fn update(
        &mut self,
        msg: Self::Message,
    ) -> ShouldRender {
        match msg {
            Msg::Ignore => (),
            Msg::WsAction(action) => match action {
                WsAction::Connect => {
                    let callback = self.link.callback(|Json(data)| Msg::WsReady(data));
                    let notification = self.link.callback(|status| match status {
                        WebSocketStatus::Opened => Msg::Ignore,
                        WebSocketStatus::Closed | WebSocketStatus::Error => {
                            Msg::WsAction(WsAction::Lost.into())
                        }
                    });
                    let task = WebSocketService::connect(
                        "wss://rse5mmis8e.execute-api.ca-central-1.amazonaws.com/dev",
                        callback,
                        notification,
                    )
                    .unwrap();
                    self.ws = Some(task);
                }
                WsAction::SendData(data) => {
                    log::info!("Sending data");
                    self.ws.as_mut().unwrap().send(Json(&data));
                }
                WsAction::Disconnect => {
                    log::info!("Disconnecting from WebSocket");
                    self.ws.take();
                }
                WsAction::Lost => {
                    log::info!("WebSocket connection lost.");
                    self.ws = None;
                }
            },
            Msg::WsReady(response) => {
                log::info!("Received message from WebSocket, {:?}", &response);
                self.lobby = response.map(|data| data.lobby).ok();
            }
            Msg::NameInputChange(new_value) => {
                self.name = new_value;
            }
            Msg::LobbyCodeInputChange(new_value) => {
                self.lobby_code_input = new_value;
            }
        }
        true
    }

    fn change(
        &mut self,
        _props: Self::Properties,
    ) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <div>{ self.view_connection_status() }</div>
                <div>{ self.view_name() }</div>
                <div>{ self.view_lobby() }</div>
            </div>
        }
    }

    fn rendered(
        &mut self,
        first_render: bool,
    ) {
        if first_render {
            log::info!("Connecting to Websocket");
            self.link.send_message(Msg::WsAction(WsAction::Connect));
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    wasm_logger::init(wasm_logger::Config::default());
    App::<Model>::new().mount_to_body();
}
