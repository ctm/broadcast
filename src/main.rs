use {
    gloo_events::EventListener,
    serde::{Deserialize, Serialize},
    std::{marker::PhantomData, sync::OnceLock},
    wasm_bindgen::JsCast,
    web_sys::{BroadcastChannel, MessageEvent},
    yew::{html::Scope, prelude::*},
    yew_router::prelude::*,
};

const CHANNEL_NAME: &str = "session-server";

#[derive(Clone, Debug, Eq, PartialEq, Routable)]
pub enum Route {
    #[at("/client")]
    Client,

    #[at("/")]
    Server,
}

trait Yyy: Component {
    fn construct_processor(link: &Scope<Self>) -> impl FnMut(&ChannelMessage) + 'static;
}

struct SessionChannel<T: Yyy> {
    channel: BroadcastChannel,
    _listener: EventListener,
    component_type: PhantomData<T>,
}

fn origin() -> &'static str {
    static ORIGIN: OnceLock<String> = OnceLock::new();
    ORIGIN
        .get_or_init(|| gloo_utils::window().origin())
        .as_str()
}

impl<T: Yyy> SessionChannel<T> {
    fn new(link: &Scope<T>) -> Self {
        let channel = BroadcastChannel::new(CHANNEL_NAME).unwrap();
        let mut yyy = T::construct_processor(link);
        let _listener = EventListener::new(&channel, "message", move |event| {
            let event = event.unchecked_ref::<MessageEvent>();
            if event.origin() == origin() {
                if let Ok(f) = serde_wasm_bindgen::from_value::<ChannelMessage>(event.data()) {
                    yyy(&f);
                    event.stop_immediate_propagation();
                }
            }
        });
        Self {
            channel,
            _listener,
            component_type: PhantomData,
        }
    }

    fn send_session_id(&self) {
        todo!()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
enum ChannelMessage {
    WhatIsMySessionId,
    SessionIdIs(u64),
}

mod server {
    use super::{ChannelMessage, SessionChannel, Yyy};
    use yew::{html::Scope, prelude::*};

    pub(super) struct Server {
        channel: SessionChannel<Self>,
    }

    pub(super) enum Msg {
        SessionIdRequested,
    }

    impl Yyy for Server {
        fn construct_processor(link: &Scope<Self>) -> impl FnMut(&ChannelMessage) + 'static {
            move |message| todo!()
        }
    }

    impl Component for Server {
        type Message = Msg;
        type Properties = ();

        fn create(ctx: &Context<Self>) -> Self {
            let channel = SessionChannel::new(&ctx.link());
            Self { channel }
        }

        fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
            let Msg::SessionIdRequested = msg;
            self.channel.send_session_id();
            false
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
    use super::{ChannelMessage, SessionChannel, Yyy};
    use gloo_timers::callback::Timeout;
    use yew::{html::Scope, prelude::*};

    enum Msg {
        IdIs(u64),
        TimedOut,
    }

    enum Client {
        Trying(SessionChannel<Self>, Timeout),
        SessionId(u64),
        GaveUp,
    }

    impl Yyy for Client {
        fn construct_processor(link: &Scope<Self>) -> impl FnMut(&ChannelMessage) + 'static {
            move |message| todo!()
        }
    }

    impl Component for Client {
        type Message = Msg;
        type Properties = ();
        fn create(ctx: &Context<Self>) -> Self {
            let channel = SessionChannel::new(&ctx.link());
            let link = ctx.link().clone();
            let timeout = Timeout::new(100, move || {
                link.send_message(Msg::TimedOut);
            });
            Self::Trying(channel, timeout)
        }

        fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
            match msg {
                Msg::IdIs(id) => *self = Self::SessionId(id),
                Msg::TimedOut => *self = Self::GaveUp,
            }
            false
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

fn main() {
    yew::Renderer::<server::Server>::new().render();
}
