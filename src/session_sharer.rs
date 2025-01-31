use {
    gloo_events::EventListener,
    gloo_timers::callback::Timeout,
    serde::{Deserialize, Serialize},
    std::sync::OnceLock,
    wasm_bindgen::JsCast,
    web_sys::{BroadcastChannel, MessageEvent},
    yew::{html::Scope, prelude::*},
};

const CHANNEL_NAME: &str = "session-sharer";

type SessionId = u64;

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
pub(crate) struct IdSender(IdChannel);

impl IdSender {
    pub(crate) fn new(id: Option<SessionId>) -> Self {
        Self(IdChannel::new(Self::make_doit(id)))
    }

    pub(crate) fn update(&mut self, id: Option<SessionId>) {
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

// TODO: Make the receiver be an enum that is like super::Client in
// that it knows the session id itself.  It will then only need a
// message that it sends when it transitions from Trying to SessionId
// or GaveUp.  Then it can have a session_id method that returns an
// Option<SessionId>.

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct IdReceiver(IdChannel, Timeout);

impl IdReceiver {
    pub(crate) fn new<T>(
        link: &Scope<T>,
        rcv: fn(Option<SessionId>) -> T::Message,
        tmo_msg: T::Message,
    ) -> Self
    where
        T: Component,
        T::Message: Clone,
    {
        let channel = {
            let link = link.clone();
            IdChannel::new(move |message, _channel| {
                if let Message::Response(id) = message {
                    link.send_message(rcv(id))
                }
            })
        };
        channel.send(&Message::Query);

        let timeout = {
            let link = link.clone();
            let tmo_msg = tmo_msg.clone();
            Timeout::new(10, move || {
                link.send_message(tmo_msg.clone());
            })
        };
        Self(channel, timeout)
    }
}
