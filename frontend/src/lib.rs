use wasm_bindgen::prelude::*;

use anyhow::Error;
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Value};
use yew::format::Json;
use yew::prelude::*;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};

struct Model {
    link: ComponentLink<Self>,
    lobby: Option<Lobby>,
    ws: Option<WebSocketTask>,
}

impl Model {
    fn view_lobby(&self) -> Html {
        if let Some(lobby) = &self.lobby {
            html! {
                <p>{ lobby.code.clone() }</p>
            }
        } else {
            html! {
                <p>{ "Not in a lobby." }</p>
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
    code: String,
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
            lobby: None,
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
                <button
                    class="w-32 m-4 bg-blue-200 hover:bg-blue-300 rounded-lg shadow-md"
                    disabled=self.ws.is_none()
                    onclick=self.link.callback(|_| {
                        let create_lobby_message = json!({
                          "action": "hearts",
                          "type": "lobby_action_create",
                          "name": "Host",  // TODO make this a field they can edit
                        });
                        Msg::WsAction(WsAction::SendData(create_lobby_message))
                    })
                >
                    { "Create lobby" }
                </button>
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
