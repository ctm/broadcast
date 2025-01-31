#![deny(clippy::unwrap_used, clippy::expect_used)]

use {
    session_sharer::{IdSender, Passthrough, Source},
    yew::prelude::*,
    yew_router::prelude::*,
};

mod session_sharer;

type SessionId = u64;

#[derive(Clone, Eq, PartialEq, Routable)]
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
        log::error,
        yew::prelude::*,
    };

    #[allow(dead_code)]
    pub(super) struct Server {
        session_id: SessionId,
        sender: Option<IdSender>,
        update_timer: Interval,
    }

    impl Component for Server {
        type Message = ();
        type Properties = ();

        fn create(ctx: &Context<Self>) -> Self {
            let session_id = Default::default();

            let sender = IdSender::new(None)
                .inspect_err(|e| error!("IdSender::new failed: {e}"))
                .ok();

            let update_timer = {
                let link = ctx.link().clone();
                Interval::new(1_000, move || link.send_message(()))
            };

            Self {
                session_id,
                sender,
                update_timer,
            }
        }

        fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
            self.session_id += 1;
            if self.session_id > 2 {
                if let Some(sender) = &mut self.sender {
                    sender.update(Some(self.session_id));
                }
            }
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
    use super::{Passthrough, Source};
    use yew::prelude::*;

    #[derive(Clone)]
    pub(super) enum Msg {
        Passthrough(Passthrough),
    }

    pub(super) struct Client {
        source: Source,
    }

    impl Component for Client {
        type Message = Msg;
        type Properties = ();
        fn create(ctx: &Context<Self>) -> Self {
            Self {
                source: Source::new(ctx.link(), Msg::Passthrough),
            }
        }

        fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
            match msg {
                Msg::Passthrough(pass) => self.source.update(pass),
            }
            true
        }

        fn view(&self, _ctx: &Context<Self>) -> Html {
            html! {
                {
                    match self.source.session_id() {
                        Some(id) => html! { id },
                        None => html! { "Not Logged In" },
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
