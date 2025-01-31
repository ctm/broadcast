use {
    super::SessionId,
    gloo_events::EventListener,
    gloo_timers::callback::Timeout,
    serde::{Deserialize, Serialize},
    std::sync::OnceLock,
    wasm_bindgen::JsCast,
    web_sys::{BroadcastChannel, MessageEvent},
    yew::{html::Scope, prelude::*},
};

const CHANNEL_NAME: &str = "session-sharer";

fn origin() -> &'static str {
    static ORIGIN: OnceLock<String> = OnceLock::new();
    ORIGIN
        .get_or_init(|| gloo_utils::window().origin())
        .as_str()
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
enum Message {
    Query,
    Response(Option<SessionId>),
}

#[derive(Debug)]
struct IdChannel {
    channel: BroadcastChannel,
    _listener: EventListener,
}

impl IdChannel {
    fn new(doit: impl Fn(Message, &BroadcastChannel) + 'static) -> Self {
        let channel = BroadcastChannel::new(CHANNEL_NAME).unwrap();
        let _listener = Self::mk_listener(&channel, doit);
        Self { channel, _listener }
    }

    fn update_listener(&mut self, doit: impl Fn(Message, &BroadcastChannel) + 'static) {
        self._listener = Self::mk_listener(&self.channel, doit);
    }

    fn send(&self, message: &Message) {
        let message = serde_wasm_bindgen::to_value(message).unwrap();
        self.channel.post_message(&message).unwrap();
    }

    fn mk_listener(
        channel: &BroadcastChannel,
        doit: impl Fn(Message, &BroadcastChannel) + 'static,
    ) -> EventListener {
        let channel_clone = channel.clone();
        EventListener::new(channel, "message", move |event| {
            let event = event.unchecked_ref::<MessageEvent>();
            if event.origin() == origin() {
                if let Ok(f) = serde_wasm_bindgen::from_value::<Message>(event.data()) {
                    doit(f, &channel_clone);
                }
            }
        })
    }
}

#[allow(dead_code)]
pub(super) struct IdSender(IdChannel);

impl IdSender {
    pub(super) fn new(id: Option<SessionId>) -> Self {
        Self(IdChannel::new(Self::make_doit(id)))
    }

    pub(super) fn update(&mut self, id: Option<SessionId>) {
        self.0.update_listener(Self::make_doit(id));
    }

    fn make_doit(id: Option<SessionId>) -> impl Fn(Message, &BroadcastChannel) + 'static {
        let to_send = serde_wasm_bindgen::to_value(&Message::Response(id)).unwrap();
        move |message, channel| {
            if message == Message::Query {
                channel.post_message(&to_send).unwrap();
            }
        }
    }
}

#[allow(dead_code)]
struct IdReceiver(IdChannel, Timeout);

impl IdReceiver {
    fn new<T: Component>(link: &Scope<T>, pass: fn(Passthrough) -> T::Message) -> Self {
        let channel = {
            let link = link.clone();
            IdChannel::new(move |message, _channel| {
                if let Message::Response(id) = message {
                    link.send_message(pass(Passthrough::Id(id)));
                }
            })
        };
        channel.send(&Message::Query);

        let timeout = {
            let link = link.clone();
            Timeout::new(10, move || {
                link.send_message(pass(Passthrough::TimedOut));
            })
        };
        Self(channel, timeout)
    }
}

#[derive(Clone, Copy)]
pub(super) enum Passthrough {
    Id(Option<SessionId>),
    TimedOut,
}

enum Inner {
    #[allow(dead_code)]
    Trying(IdReceiver),
    SessionId(Option<SessionId>),
    GaveUp,
}

impl Inner {
    fn new<T: Component>(link: &Scope<T>, pass: fn(Passthrough) -> T::Message) -> Self {
        Self::Trying(IdReceiver::new(link, pass))
    }

    fn session_id(&self) -> Option<SessionId> {
        use Inner::*;

        match self {
            Trying(_) | GaveUp => None,
            SessionId(id) => *id,
        }
    }

    fn update(&mut self, arg: Passthrough) {
        use Passthrough::*;

        match arg {
            Id(id) => *self = Inner::SessionId(id),
            TimedOut => *self = Inner::GaveUp,
        }
    }
}

pub(super) struct Source(Inner);

impl Source {
    pub(super) fn new<T: Component>(link: &Scope<T>, pass: fn(Passthrough) -> T::Message) -> Self {
        Self(Inner::new(link, pass))
    }

    pub(super) fn session_id(&self) -> Option<SessionId> {
        self.0.session_id()
    }

    pub(super) fn update(&mut self, arg: Passthrough) {
        self.0.update(arg)
    }
}
