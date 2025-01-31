use {
    session_sharer::{IdReceiver, IdSender},
    yew::prelude::*,
    yew_router::prelude::*,
};

mod session_sharer;

type SessionId = u64;

#[derive(Clone, Debug, Eq, PartialEq, Routable)]
pub enum Route {
    #[at("/client")]
    Client,

    #[at("/")]
    Server,
}

mod server {
    use {
        super::{IdSender, SessionId},
        gloo_timers::callback::Interval,
        yew::prelude::*,
    };

    #[allow(dead_code)]
    pub(super) struct Server {
        session_id: SessionId,
        sender: IdSender,
        update_timer: Interval,
    }

    impl Component for Server {
        type Message = ();
        type Properties = ();

        fn create(ctx: &Context<Self>) -> Self {
            let session_id = 42; // Default::default();
            let update_timer = {
                let link = ctx.link().clone();
                Interval::new(1_000, move || link.send_message(()))
            };

            Self {
                session_id,
                sender: IdSender::new(Some(session_id)),
                update_timer,
            }
        }

        fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
            self.session_id += 1;
            log::info!("session_id: {}", self.session_id);
            self.sender.update(Some(self.session_id));
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
    use super::{IdReceiver, SessionId};
    use yew::prelude::*;

    #[derive(Clone)]
    pub(super) enum Msg {
        IdIs(Option<SessionId>),
        TimedOut,
    }

    #[derive(Debug)]
    pub(super) enum Client {
        #[allow(dead_code)]
        Trying(IdReceiver),
        SessionId(Option<u64>),
        GaveUp,
    }

    impl Component for Client {
        type Message = Msg;
        type Properties = ();
        fn create(ctx: &Context<Self>) -> Self {
            Self::Trying(IdReceiver::new(ctx.link(), Msg::IdIs, Msg::TimedOut))
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
                        Self::SessionId(Some(id)) => html! { id },
                        Self::SessionId(None) => html! { "Not Logged In" },
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
    let _ = console_log::init_with_level(log::Level::Trace);
    yew::Renderer::<App>::new().render();
}
