#![recursion_limit = "512"]

use wasm_bindgen::prelude::*;

use anyhow::Error;
use serde_json::{json, Value};
use yew::format::Json;
use yew::prelude::*;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};

struct OutOfLobbyData {
    lobby_code_input: String,
    name_input: String,
}

enum Scene {
    OutOfLobby(OutOfLobbyData),
    Lobby,
}

struct Requests {
    joining_lobby: bool,
    creating_lobby: bool,
}

struct Model {
    link: ComponentLink<Self>,
    scene: Scene,

    ws: Option<WebSocketTask>,

    lobby_chat_input: String,

    requests: Requests,

    name: String,
    lobby: Option<messages::Lobby>,
    chat_messages: Vec<messages::LobbyMessageResponse>,
}

impl Model {
    fn scene_out_of_lobby(&self) -> Html {
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
            <>
                <div>
                    <h1 class="text-xl text-center pb-4">{"Hearts app"}</h1>

                    <p>{"Play a game of the trick based card game hearts with your friends."}</p>
                    <p>{"This is a work in progress by Douglas Anderson."}</p>

                </div>

                <div>
                    <p class="text-center">{ "Not currently in a lobby." }</p>
                </div>

                <div>
                    <h2>{"Step 1"}</h2>

                    <div>
                        <label for="name-input">{"Lobby code:"}</label>
                        <input
                            id="name-input"
                            value=&self.name
                            oninput=self.link.callback(|e: InputData| Msg::NameInputChange(e.value))
                            placeholder="Pick a name your friends will see."
                            class="m-4 focus:ring-2 focus:ring-blue-600 rounded-lg shadow-md"
                        />
                    </div>

                </div>

                <div class="flex flex-col">
                    <h2>{"Step 2"}</h2>
                    <div class="p-2">
                        <label for="lobby-code-input">{"Lobby code:"}</label>
                        <input
                            id="lobby-code-input"
                            value=&self.lobby_code_input
                            oninput=self.link.callback(|e: InputData| Msg::LobbyCodeInputChange(e.value))
                            placeholder="Get this from a friend"
                            class="m-4 p-2 focus:ring-2 focus:ring-blue-600 rounded-lg shadow-md"
                        />
                    </div>
                    <div class="flex flex-row">
                        <button
                            class="w-48 mx-auto my-4 py-2 disabled:opacity-50 bg-blue-200 hover:bg-blue-300 rounded-lg shadow-md"
                            disabled={self.requests.creating_lobby || self.requests.joining_lobby}
                            onclick=self.link.callback(move |_| {
                                Msg::WsAction(WsAction::SendLobbyJoin(join_lobby_message.clone()))
                            })
                        >
                            { if self.requests.joining_lobby { self.view_loading_spinner() } else { html!{} } }
                            { "Join lobby" }
                        </button>
                    </div>
                </div>

                <div>
                    <hr />
                    <p class="text-center">{ "Or" }</p>
                </div>

                <div class="flex flex-row">
                    <button
                        class="w-48 mx-auto my-4 py-2 disabled:opacity-50 bg-blue-200 hover:bg-blue-300 rounded-lg shadow-md"
                        disabled={self.requests.creating_lobby || self.requests.joining_lobby}
                        onclick=self.link.callback(move |_| {
                            Msg::WsAction(WsAction::SendLobbyCreate(create_lobby_message.clone()))
                        })
                    >
                        { if self.requests.creating_lobby { self.view_loading_spinner() } else { html!{} } }
                        { "Create lobby" }
                    </button>
                </div>

            </>
        }
    }

    fn scene_lobby(&self) -> Html {
        html! {
            <div>
                <div>{ self.view_lobby() }</div>
                <div>{ self.view_messages() }</div>
                <div>{ self.view_connection_status() }</div>
            </div>
        }
    }

    fn view_loading_spinner(&self) -> Html {
        html! {
            <svg class="animate-spin inline-block mr-3 h-5 w-5 text-grey-500" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
        }
    }

    fn view_messages(&self) -> Html {
        if let Some(lobby) = &self.lobby {
            let send_lobby_message = json!({
              "action": "send",
              "lobby_code": lobby.id.clone(),
              "body": self.lobby_chat_input.clone(),
            });
            html! {
                <div>
                    <input
                        value=&self.lobby_chat_input
                        oninput=self.link.callback(|e: InputData| Msg::LobbyChatInputChange(e.value))
                        placeholder="Say something.."
                        class="m-4 focus:ring-2 focus:ring-blue-600 rounded-lg shadow-md"
                    />
                    <button
                        class="w-32 m-4 disabled:opacity-50 bg-blue-200 hover:bg-blue-300 rounded-lg shadow-md"
                        disabled={self.lobby_chat_input.is_empty()}
                        onclick=self.link.callback(move |_| {
                            Msg::WsAction(WsAction::SendLobbyMessage(send_lobby_message.clone()))
                        })
                    >
                        { "Send" }
                    </button>
                    <div>
                    {
                        for self.chat_messages.iter().map(|m| {
                            html! {
                                <div class="flex">
                                    <div class="m4 bg-green-200 rounded-lg">
                                        { m.name.clone() }
                                    </div>
                                    { m.body.clone() }
                                </div>
                            }
                        })
                    }
                    </div>
                </div>
            }
        } else {
            html! {}
        }
    }

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
            html! {}
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
    SendLobbyMessage(Value),
    SendLobbyCreate(Value),
    SendLobbyJoin(Value),
    Disconnect,
    Lost,
}

enum Msg {
    Ignore,
    WsAction(WsAction),
    WsReady(Result<messages::Message, Error>),
    NameInputChange(String),
    LobbyCodeInputChange(String),
    LobbyChatInputChange(String),
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
            scene: Scene::OutOfLobby,
            name: "".to_owned(),
            lobby: None,
            lobby_code_input: "".to_owned(),
            lobby_chat_input: "".to_owned(),
            requests: Requests {
                creating_lobby: false,
                joining_lobby: false,
            },
            chat_messages: vec![],
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
                WsAction::SendLobbyCreate(data) => {
                    log::info!("Sending data");
                    self.requests.creating_lobby = true;
                    self.ws.as_mut().unwrap().send(Json(&data));
                }
                WsAction::SendLobbyJoin(data) => {
                    log::info!("Sending data");
                    self.requests.joining_lobby = true;
                    self.ws.as_mut().unwrap().send(Json(&data));
                }
                WsAction::SendLobbyMessage(data) => {
                    log::info!("Sending data");
                    // Optimistic UI. We assume that anything we send will be received
                    self.chat_messages.push(messages::LobbyMessageResponse {
                        name: self.name.clone(),
                        body: self.lobby_chat_input.clone(),
                    });
                    self.lobby_chat_input = "".to_owned();
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
                match response.expect("Received bad message") {
                    messages::Message::LobbyActionCreateResponse(m) => {
                        self.requests.creating_lobby = false;
                        self.lobby = Some(m.lobby);
                    }
                    messages::Message::LobbyActionJoinResponse(m) => {
                        self.requests.joining_lobby = false;
                        self.lobby = Some(m.lobby);
                    }
                    messages::Message::LobbyMessageResponse(m) => {
                        self.chat_messages.push(m);
                    }
                    _ => {
                        log::error!("Received non-matched message");
                    }
                }
            }
            Msg::NameInputChange(new_value) => {
                self.name = new_value;
            }
            Msg::LobbyCodeInputChange(new_value) => {
                self.lobby_code_input = new_value;
            }
            Msg::LobbyChatInputChange(new_value) => {
                self.lobby_chat_input = new_value;
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
        let scene = match self.scene {
            Scene::OutOfLobby => self.scene_out_of_lobby(),
            Scene::Lobby => self.scene_lobby(),
        };

        html! {
            <div class="flex flex-col justify-between w-auto h-screen bg-green-50">
                {scene}
                <footer>{ self.view_connection_status() }</footer>
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
