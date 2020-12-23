use wasm_bindgen::prelude::*;

use anyhow::Error;
use serde_derive::{Deserialize, Serialize};
use yew::format::Json;
use yew::prelude::*;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};

struct Model {
    link: ComponentLink<Self>,
    data: Option<u32>,
    ws: Option<WebSocketTask>,
}

impl Model {
    fn view_data(&self) -> Html {
        if let Some(value) = self.data {
            html! {
                <p>{ value }</p>
            }
        } else {
            html! {
                <p>{ "Data hasn't fetched yet." }</p>
            }
        }
    }

    fn view_connection_status(&self) -> Html {
        if self.ws.is_some() {
            html! {
                <>
                    <p>{ "Connected" }</p>
                    <button
                        class="w-32 m-4 bg-red-200 hover:bg-red-300 rounded-lg shadow-md"
                        onclick=self.link.callback(|_| Msg::WsAction(WsAction::Disconnect))
                    >
                        { "Disconnect" }
                    </button>
                </>
            }
        } else {
            html! {
                <>
                    <p>{ "Not connected" }</p>
                    <button
                        class="w-32 m-4 bg-blue-200 hover:bg-blue-300 rounded-lg shadow-md"
                        onclick=self.link.callback(|_| Msg::WsAction(WsAction::Connect))
                    >
                        { "Connect" }
                    </button>
                </>
            }
        }
    }
}

enum WsAction {
    Connect,
    SendData,
    Disconnect,
    Lost,
}

enum Msg {
    Ignore,
    WsAction(WsAction),
    WsReady(Result<WsResponse, Error>),
}

/// This type is used as a request which sent to websocket connection.
#[derive(Serialize, Debug)]
struct WsRequest {
    value: u32,
}

/// This type is an expected response from a websocket connection.
#[derive(Deserialize, Debug)]
pub struct WsResponse {
    value: u32,
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
            data: None,
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
                WsAction::SendData => {
                    let request = WsRequest { value: 321 };
                    self.ws.as_mut().unwrap().send(Json(&request));
                }
                WsAction::Disconnect => {
                    self.ws.take();
                }
                WsAction::Lost => {
                    self.ws = None;
                }
            },
            Msg::WsReady(response) => self.data = response.map(|data| data.value).ok(),
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
                    onclick=self.link.callback(|_| Msg::WsAction(WsAction::SendData))
                >
                    { "Create lobby" }
                </button>
                <div>{ self.view_data() }</div>
            </div>
        }
    }

    fn rendered(
        &mut self,
        first_render: bool,
    ) {
        if first_render {
            self.link.send_message(Msg::WsAction(WsAction::Connect));
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    App::<Model>::new().mount_to_body();
}
