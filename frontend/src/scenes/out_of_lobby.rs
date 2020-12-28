use serde_json::json;
use yew::prelude::*;

use crate::components::loading_spinner::LoadingSpinner;

#[derive(Clone, Debug, PartialEq, Properties)]
pub struct Props {
    pub request_joining_lobby: bool,
    pub request_creating_lobby: bool,
}

pub struct OutOfLobby {
    link: ComponentLink<Self>,
    props: Props,

    // on_set_name: Callback<String>,
    name: String,
    lobby_code_input: String,
    name_input: String,
}

impl OutOfLobby {
    fn view_name_input(&self) -> Html {
        html! {
            <div class="flex flex-col p-2 bg-white shadow-lg">
                <h2>{"Step 1"}</h2>

               <div class="flex flex-row">
                    <label
                        for="name-input"
                        class="self-center"
                    >
                        {"Name:"}
                    </label>
                    <input
                        id="name-input"
                        value=&self.name_input
                        oninput=self.link.callback(|e: InputData| Msg::NameInputChange(e.value))
                        placeholder="Pick a name your friends will see."
                        class="m-4 p-2 flex-grow focus:ring-2 focus:ring-blue-600 rounded-lg shadow-md"
                    />
                </div>

                <div class="flex flex-row">
                    <button
                        class="w-48 mx-auto my-4 py-2 disabled:opacity-50 bg-blue-200 hover:bg-blue-300 rounded-lg shadow-md"
                        onclick=self.link.callback(move |_| { Msg::NameInputConfirm })
                    >
                        { "OK" }
                    </button>
                </div>

            </div>
        }
    }

    fn view_create_or_join(&self) -> Html {
        let create_lobby_message = json!({
          "action": "hearts",
          "type": "lobby_action_create",
          "name": self.name_input.clone(),
        });
        let join_lobby_message = json!({
          "action": "hearts",
          "type": "lobby_action_join",
          "lobby_code": self.lobby_code_input.clone(),
          "name": self.name_input.clone(),
        });
        html! {
            <div class="flex flex-col p-2 bg-white shadow-lg">
                <h2>{"Step 2"}</h2>

                <div class="flex flex-row">
                    <label
                        for="lobby_code-input"
                        class="self-center"
                    >
                        {"Lobby code:"}
                    </label>
                    <input
                        id="lobby-code-input"
                        value=&self.lobby_code_input
                        oninput=self.link.callback(|e: InputData| Msg::LobbyCodeInputChange(e.value))
                        placeholder="Get this from a friend"
                        class="m-4 p-2 flex-grow focus:ring-2 focus:ring-blue-600 rounded-lg shadow-md"
                    />
                </div>
                <div class="flex flex-row">
                    <button
                        class="w-48 mx-auto my-4 py-2 disabled:opacity-50 bg-blue-200 hover:bg-blue-300 rounded-lg shadow-md"
                        disabled={self.props.request_creating_lobby || self.props.request_joining_lobby}
                        onclick=self.link.callback(move |_| {
                            Msg::Ignore
                            // Msg::WsAction(WsAction::SendLobbyJoin(join_lobby_message.clone()))
                        })
                    >
                        { if self.props.request_joining_lobby { html!{ <LoadingSpinner /> } } else { html!{} } }
                        { "Join lobby" }
                    </button>
                </div>

                <div>
                    <hr />
                    <p class="text-center">{ "Or" }</p>
                </div>

                <button
                    class="w-48 mx-auto my-4 py-2 disabled:opacity-50 bg-blue-200 hover:bg-blue-300 rounded-lg shadow-md"
                    disabled={self.props.request_creating_lobby || self.props.request_joining_lobby}
                    onclick=self.link.callback(move |_| {
                            Msg::Ignore
                        // Msg::WsAction(WsAction::SendLobbyCreate(create_lobby_message.clone()))
                    })
                >
                    { if self.props.request_creating_lobby { html!{ <LoadingSpinner />} } else { html!{} } }
                    { "Create lobby" }
                </button>
            </div>
        }
    }
}

enum Msg {
    Ignore,
    NameInputConfirm,
    NameInputChange(String),
    LobbyCodeInputChange(String),
}

impl Component for OutOfLobby {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            props,
            name: "".to_owned(),
            lobby_code_input: "".to_owned(),
            name_input: "".to_owned(),
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Ignore => {}
            Msg::NameInputChange(new_value) => {
                self.name_input = new_value;
            }
            Msg::LobbyCodeInputChange(new_value) => {
                self.lobby_code_input = new_value;
            }
            Msg::NameInputConfirm => {
                self.name = self.name_input.clone();
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
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

                { if self.name.is_empty() { self.view_name_input() } else { self.view_create_or_join() } }

            </>
        }
    }
}
