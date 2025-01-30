use {gloo_events::EventListener, web_sys::BroadcastChannel, yew_router::prelude::*};

const CHANNEL_NAME: &str = "session-server";

#[derive(Clone, Debug, Eq, PartialEq, Routable)]
pub enum Route {
    #[at("/client")]
    Client,

    #[at("/")]
    Server,
}

mod server {
    use super::{BroadcastChannel, EventListener, CHANNEL_NAME};
    use yew::prelude::*;

    pub(super) struct Server {
        channel: BroadcastChannel,
        _listener: EventListener,
    }

    pub(super) enum Msg {}

    impl Component for Server {
        type Message = Msg;
        type Properties = ();

        fn create(_ctx: &Context<Self>) -> Self {
            let channel = BroadcastChannel::new(CHANNEL_NAME).unwrap();
            let _listener = EventListener::new(&channel, "message", move |event| todo!());
            Self { channel, _listener }
        }

        fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
            true
        }

        fn view(&self, _ctx: &Context<Self>) -> Html {
            html! {
                <main>
                    <img class="logo" src="https://yew.rs/img/logo.png" alt="Yew logo" />
                    <h1>{ "Hello World!" }</h1>
                    <span class="subtitle">{ "from Yew with " }<i class="heart" /></span>
                    </main>
            }
        }
    }
}

mod client {
    use super::{EventListener, CHANNEL_NAME};

    struct Client {
        _channel: EventListener,
    }
}

fn main() {
    yew::Renderer::<server::Server>::new().render();
}
