use {
    gloo_events::EventListener,
    serde::{Deserialize, Serialize},
    std::{marker::PhantomData, sync::OnceLock},
    wasm_bindgen::JsCast,
    web_sys::{BroadcastChannel, MessageEvent},
    yew::{html::Scope, prelude::*},
    yew_router::prelude::*,
};

mod session_sharer;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
enum Message {
    WhatIsMySessionId,
    SessionIdIs(u64),
}


struct SessionServer {
    channel: BroadcastChannel,
    _listener: EventListener,
}

impl SessionServer {
    fn new<T: Component>(msg: T::Message) -> Self {
        todo!()
    }
    
}



const CHANNEL_NAME: &str = "session-server";

#[derive(Clone, Debug, Eq, PartialEq, Routable)]
pub enum Route {
    #[at("/client")]
    Client,

    #[at("/")]
    Server,
}

trait ConstructsProcessor: Component {
    fn construct_processor(link: &Scope<Self>) -> impl FnMut(&ChannelMessage) + 'static;
}

#[derive(Debug)]
struct SessionChannel<T: ConstructsProcessor> {
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

impl<T: ConstructsProcessor> SessionChannel<T> {
    fn new(link: &Scope<T>) -> Self {
        let channel = BroadcastChannel::new(CHANNEL_NAME).unwrap();
        let mut process = T::construct_processor(link);
        let _listener = EventListener::new(&channel, "message", move |event| {
            let event = event.unchecked_ref::<MessageEvent>();
            if event.origin() == origin() {
                if let Ok(f) = serde_wasm_bindgen::from_value::<ChannelMessage>(event.data()) {
                    log::info!("f: {f:?}");
                    process(&f);
                    // event.stop_immediate_propagation();
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
        let message = serde_wasm_bindgen::to_value(&ChannelMessage::SessionIdIs(42)).unwrap();
        self.channel.post_message(&message).unwrap();
    }

    fn request_session_id(&self) {
        let message = serde_wasm_bindgen::to_value(&ChannelMessage::WhatIsMySessionId).unwrap();
        self.channel.post_message(&message).unwrap();
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
enum ChannelMessage {
    WhatIsMySessionId,
    SessionIdIs(u64),
}

mod server {
    use super::{ChannelMessage, SessionChannel, ConstructsProcessor};
    use yew::{html::Scope, prelude::*};

    pub(super) struct Server {
        channel: SessionChannel<Self>,
    }

    pub(super) enum Msg {
        SessionIdRequested,
    }

    impl ConstructsProcessor for Server {
        fn construct_processor(link: &Scope<Self>) -> impl FnMut(&ChannelMessage) + 'static {
            let link = link.clone();
            move |message| if *message == ChannelMessage::WhatIsMySessionId {
                link.send_message(Msg::SessionIdRequested);
            }
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
    use super::{ChannelMessage, SessionChannel, ConstructsProcessor};
    use gloo_timers::callback::Timeout;
    use yew::{html::Scope, prelude::*};

    pub(super) enum Msg {
        IdIs(u64),
        TimedOut,
    }

    #[derive(Debug)]
    pub(super) enum Client {
        Trying(SessionChannel<Self>, Timeout),
        SessionId(u64),
        GaveUp,
    }

    impl ConstructsProcessor for Client {
        fn construct_processor(link: &Scope<Self>) -> impl FnMut(&ChannelMessage) + 'static {
            let link = link.clone();
            move |message| {
                if let ChannelMessage::SessionIdIs(id)  = message {
                    link.send_message(Msg::IdIs(*id));
                }
            }
        }
    }

    impl Component for Client {
        type Message = Msg;
        type Properties = ();
        fn create(ctx: &Context<Self>) -> Self {
            let channel = SessionChannel::new(&ctx.link());
            channel.request_session_id();
            let timeout = {
                let link = ctx.link().clone();

                Timeout::new(10, move || {
                    link.send_message(Msg::TimedOut);
                })
            };
            Self::Trying(channel, timeout)
        }

        fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
            match msg {
                Msg::IdIs(id) => *self = Self::SessionId(id),
                Msg::TimedOut => *self = Self::GaveUp,
            }
            true
        }

        fn view(&self, _ctx: &Context<Self>) -> Html {
            html! {
                {
                    match self {
                        Self::Trying(..) => html!{},
                        Self::SessionId(id) => html! { id},
                        Self::GaveUp => html! { "Gave Up" },
                    }
                }
            }
        }
    }
}

fn switch(route: Route) -> Html {
    use Route::*;

    match route {
        Client => html! { <client::Client /> },
        Server => html! { <server::Server /> },
    }
}


#[function_component(App)]
pub fn app() -> Html {
    html! {
        <main>
            <BrowserRouter>
                <Switch<Route> render={switch} />
            </BrowserRouter>
        </main>
    }
}

fn main() {
    console_log::init_with_level(log::Level::Trace);
    yew::Renderer::<App>::new().render();
}
