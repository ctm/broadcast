use {
    session_sharer::{IdReceiver, IdSender},
    yew::prelude::*,
    yew_router::prelude::*,
};

mod session_sharer;

#[derive(Clone, Debug, Eq, PartialEq, Routable)]
pub enum Route {
    #[at("/client")]
    Client,

    #[at("/")]
    Server,
}

mod server {
    use super::IdSender;
    use yew::prelude::*;

    pub(super) struct Server {
        sender: IdSender,
    }

    #[derive(Clone, Copy)]
    pub(super) enum Msg {
        SessionIdRequested,
    }

    impl Component for Server {
        type Message = Msg;
        type Properties = ();

        fn create(ctx: &Context<Self>) -> Self {
            let sender = IdSender::new(ctx.link(), Msg::SessionIdRequested);
            Self { sender }
        }

        fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
            let Msg::SessionIdRequested = msg;
            self.sender.send_id(42);
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
    use super::IdReceiver;
    use yew::prelude::*;

    #[derive(Clone)]
    pub(super) enum Msg {
        IdIs(u64),
        TimedOut,
    }

    #[derive(Debug)]
    pub(super) enum Client {
        Trying(IdReceiver),
        SessionId(u64),
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
    let _ = console_log::init_with_level(log::Level::Trace);
    yew::Renderer::<App>::new().render();
}
