use {
    super::SessionId,
    gloo_events::EventListener,
    gloo_timers::callback::Timeout,
    log::error,
    serde::{Deserialize, Serialize},
    std::sync::OnceLock,
    thiserror::Error,
    wasm_bindgen::{JsCast, JsValue},
    web_sys::{BroadcastChannel, MessageEvent},
    yew::{html::Scope, prelude::*},
};

const CHANNEL_NAME: &str = "session-sharer";

#[derive(Debug, Error)]
pub(super) enum Error {
    #[error("BroadcastChannel::new failed: {0:?}")]
    BroadcastChannelNew(JsValue),

    #[error("to_value failed: {0:?}")]
    ToValueFailed(#[from] serde_wasm_bindgen::Error),

    #[error("post_message failed: {0:?}")]
    PostMessage(JsValue),
}

impl Error {
    fn new_broadcast_channel(e: JsValue) -> Self {
        Self::BroadcastChannelNew(e)
    }
}

pub(super) type Result<O> = std::result::Result<O, Error>;

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

impl TryFrom<&Message> for JsValue {
    type Error = Error;

    fn try_from(message: &Message) -> Result<JsValue> {
        Ok(serde_wasm_bindgen::to_value(message)?)
    }
}

#[derive(Debug)]
struct IdChannel {
    channel: BroadcastChannel,
    _listener: EventListener,
}

impl IdChannel {
    fn new(doit: impl Fn(Message, &BroadcastChannel) + 'static) -> Result<Self> {
        let channel = BroadcastChannel::new(CHANNEL_NAME).map_err(Error::new_broadcast_channel)?;
        let _listener = Self::mk_listener(&channel, doit);
        Ok(Self { channel, _listener })
    }

    fn update_listener(&mut self, doit: impl Fn(Message, &BroadcastChannel) + 'static) {
        self._listener = Self::mk_listener(&self.channel, doit);
    }

    fn send(&self, message: &Message) -> Result<()> {
        self.channel
            .post_message(&message.try_into()?)
            .map_err(Error::PostMessage)
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
    pub(super) fn new(id: Option<SessionId>) -> Result<Self> {
        Ok(Self(IdChannel::new(Self::make_doit(id)?)?))
    }

    pub(super) fn update(&mut self, id: Option<SessionId>) -> Result<()> {
        self.0.update_listener(Self::make_doit(id)?);
        Ok(())
    }

    fn make_doit(id: Option<SessionId>) -> Result<impl Fn(Message, &BroadcastChannel) + 'static> {
        let to_send: JsValue = (&Message::Response(id)).try_into()?;
        Ok(move |message, channel: &BroadcastChannel| {
            if message == Message::Query {
                let _ = channel
                    .post_message(&to_send)
                    .inspect_err(|e| error!("post_message(Response({id:?}) failed: {e:?}"));
            }
        })
    }
}

#[allow(dead_code)]
struct IdReceiver(IdChannel, Timeout);

impl IdReceiver {
    fn new<T: Component>(link: &Scope<T>, pass: fn(Passthrough) -> T::Message) -> Result<Self> {
        let channel = {
            let link = link.clone();
            IdChannel::new(move |message, _channel| {
                if let Message::Response(id) = message {
                    link.send_message(pass(Passthrough::Id(id)));
                }
            })?
        };
        channel.send(&Message::Query)?;

        let timeout = {
            let link = link.clone();
            Timeout::new(10, move || {
                link.send_message(pass(Passthrough::TimedOut));
            })
        };
        Ok(Self(channel, timeout))
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum Passthrough {
    Id(Option<SessionId>),
    TimedOut,
}

impl Passthrough {
    pub(super) fn timed_out(&self) -> bool {
        *self == Passthrough::TimedOut
    }
}

enum Inner {
    #[allow(dead_code)]
    Trying(Result<IdReceiver>),
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
