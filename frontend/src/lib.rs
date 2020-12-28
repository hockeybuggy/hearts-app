#![recursion_limit = "512"]

use wasm_bindgen::prelude::*;

use anyhow::Error;
use serde_json::Value;
use yew::format::Json;
use yew::prelude::*;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};
use yew_router::{route::Route, switch::Permissive};

mod components;

mod scenes;
use scenes::{lobby::Lobby, out_of_lobby::OutOfLobby, page_not_found::PageNotFound};
mod switch;
use switch::{AppAnchor, AppRoute, AppRouter, PublicUrlSwitch};

struct Requests {
    joining_lobby: bool,
    creating_lobby: bool,
}

struct Model {
    link: ComponentLink<Self>,

    ws: Option<WebSocketTask>,

    lobby_chat_input: String,

    requests: Requests,

    lobby: Option<messages::Lobby>,
    chat_messages: Vec<messages::LobbyMessageResponse>,
}

impl Model {
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
    LobbyChatInputChange(String),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            lobby: None,
            lobby_chat_input: "".to_owned(),
            requests: Requests {
                creating_lobby: false,
                joining_lobby: false,
            },
            chat_messages: vec![],
            ws: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
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
                        name: "wat".to_owned(),
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
            Msg::LobbyChatInputChange(new_value) => {
                self.lobby_chat_input = new_value;
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&'static self) -> Html {
        html! {
            <div class="w-auto h-screen bg-green-50">
                <div class="flex flex-col h-screen justify-between container mx-auto">
                    <main>
                        <AppRouter
                            render=AppRouter::render(|switch: PublicUrlSwitch| {
                                match switch.route() {
                                    AppRoute::Lobby(lobby_code) => {
                                        html! {
                                            <Lobby
                                                lobby_code=lobby_code
                                                lobby=&self.lobby
                                                chat_messages=&self.chat_messages
                                            />
                                        }
                                    }
                                    AppRoute::Home => {
                                        html! {
                                            <OutOfLobby
                                                request_joining_lobby=&self.requests.joining_lobby
                                                request_creating_lobby=&self.requests.creating_lobby
                                            />
                                        }
                                    }
                                    AppRoute::PageNotFound(Permissive(route)) => {
                                        html! { <PageNotFound route=route /> }
                                    }
                                }
                            })
                            redirect=AppRouter::redirect(|route: Route| {
                                AppRoute::PageNotFound(Permissive(Some(route.route))).into_public()
                            })
                        />
                    </main>
                    <footer>{ self.view_connection_status() }</footer>
                </div>
            </div>
        }
    }

    fn rendered(&mut self, first_render: bool) {
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
