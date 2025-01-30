use {
    gloo_events::EventListener,
    std::marker::PhantomData,
    web_sys::BroadcastChannel,
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

trait TransformChannelMessage: Component {
    fn transform_channel_message(message: &ChannelMessage) -> Self::Message;
}

struct SessionChannel<T: Component> {
    channel: BroadcastChannel,
    _listener: EventListener,
    component_type: PhantomData<T>,
}

impl<T: TransformChannelMessage> SessionChannel<T> {
    fn new(receive: ChannelMessage, link: &Scope<T>) -> Self {
        let channel = BroadcastChannel::new(CHANNEL_NAME).unwrap();
        let _listener = EventListener::new(&channel, "message", move |event| todo!());
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

#[derive(Clone, Copy, Debug)]
enum ChannelMessage {
    WhatIsMySessionId,
    SessionIdIs(u64),
}

mod server {
    use super::{ChannelMessage, SessionChannel};
    use crate::TransformChannelMessage;
    use yew::prelude::*;

    pub(super) struct Server {
        channel: SessionChannel<Self>,
    }

    pub(super) enum Msg {
        SessionIdRequested,
    }

    impl TransformChannelMessage for Server {
        fn transform_channel_message(_message: &ChannelMessage) -> Self::Message {
            Msg::SessionIdRequested
        }
    }

    impl Component for Server {
        type Message = Msg;
        type Properties = ();

        fn create(ctx: &Context<Self>) -> Self {
            let channel = SessionChannel::new(ChannelMessage::WhatIsMySessionId, &ctx.link());
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
    use super::{ChannelMessage, SessionChannel};
    use crate::TransformChannelMessage;
    use gloo_timers::callback::Timeout;
    use yew::prelude::*;

    enum Msg {
        IdIs(u64),
        TimedOut,
    }

    enum Client {
        Trying(SessionChannel<Self>, Timeout),
        SessionId(u64),
        GaveUp,
    }

    impl TransformChannelMessage for Client {
        fn transform_channel_message(message: &ChannelMessage) -> Self::Message {
            if let ChannelMessage::SessionIdIs(id) = message {
                Msg::IdIs(*id)
            } else {
                unreachable!("gotL {message:?}");
            }
        }
    }

    impl Component for Client {
        type Message = Msg;
        type Properties = ();
        fn create(ctx: &Context<Self>) -> Self {
            let channel = SessionChannel::new(ChannelMessage::SessionIdIs, &ctx.link());
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
