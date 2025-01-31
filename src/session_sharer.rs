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
    Response(SessionId),
}

#[derive(Debug)]
struct IdChannel {
    channel: BroadcastChannel,
    _listener: EventListener,
}

impl IdChannel {
    fn new(doit: impl Fn(Message) + 'static) -> Self {
        let channel = BroadcastChannel::new(CHANNEL_NAME).unwrap();
        let _listener = EventListener::new(&channel, "message", move |event| {
            let event = event.unchecked_ref::<MessageEvent>();
            if event.origin() == origin() {
                if let Ok(f) = serde_wasm_bindgen::from_value::<Message>(event.data()) {
                    doit(f);
                }
            }
        });
        Self { channel, _listener }
    }

    fn send(&self, message: &Message) {
        let message = serde_wasm_bindgen::to_value(message).unwrap();
        self.channel.post_message(&message).unwrap();
    }
}

pub(crate) struct IdSender(IdChannel);

impl IdSender {
    pub(crate) fn new<T>(link: &Scope<T>, msg: T::Message) -> Self
    where
        T: Component,
        T::Message: Clone,
    {
        let link = link.clone();
        Self(IdChannel::new(move |message| {
            if message == Message::Query {
                link.send_message(msg.clone())
            }
        }))
    }

    pub(crate) fn send_id(&self, id: SessionId) {
        self.0.send(&Message::Response(id));
    }
}

#[derive(Debug)]
pub(crate) struct IdReceiver(IdChannel, Timeout);

impl IdReceiver {
    pub(crate) fn new<T>(
        link: &Scope<T>,
        rcv: fn(SessionId) -> T::Message,
        tmo_msg: T::Message,
    ) -> Self
    where
        T: Component,
        T::Message: Clone,
    {
        let channel = {
            let link = link.clone();
            IdChannel::new(move |message| {
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
