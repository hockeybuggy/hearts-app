use serde_json::json;
use yew::prelude::*;
use yewtil::NeqAssign;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub lobby_code: String,
    pub lobby: Option<messages::Lobby>,
    pub chat_messages: Vec<messages::LobbyMessageResponse>,
}

pub struct Lobby {
    link: ComponentLink<Self>,
    pub props: Props,

    lobby_chat_input: String,
}

impl Lobby {
    fn view_lobby(&self) -> Html {
        if let Some(lobby) = &self.props.lobby {
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

    fn view_messages(&self) -> Html {
        if let Some(lobby) = &self.props.lobby {
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
                            Msg::Ignore
                            // Msg::WsAction(WsAction::SendLobbyMessage(send_lobby_message.clone()))
                        })
                    >
                        { "Send" }
                    </button>
                    <div>
                    {
                        for self.props.chat_messages.iter().map(|m| {
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
}

enum Msg {
    Ignore,
    LobbyChatInputChange(String),
}

impl Component for Lobby {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            props,
            lobby_chat_input: "".to_owned(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Ignore => {}
            Msg::LobbyChatInputChange(new_value) => {
                self.lobby_chat_input = new_value;
            }
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props.neq_assign(props)
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <div>{ self.view_lobby() }</div>
                <div>{ self.view_messages() }</div>
            </div>
        }
    }
}
