use dioxus::logger::tracing::{self, Level};
use dioxus::prelude::*;
use serde::de::DeserializeOwned;
use shared::front::FrontMessage;
use shared::problem::Pid;
use shared::record::Rid;
use std::sync::LazyLock;

mod judge_status;
mod problem;
mod record;
mod submit;

static SERVER_ORIGIN: LazyLock<String> = LazyLock::new(|| {
    // #[cfg(not(debug_assertions))]
    // {
    web_sys::window().unwrap().origin()
    // }
    // #[cfg(debug_assertions)]
    // {
    //     "http://localhost:5800".to_string()
    // }
});

async fn send_message<T>(msg: FrontMessage) -> eyre::Result<T>
where
    T: DeserializeOwned,
{
    let res = reqwest::Client::new()
        .post(format!("{}/api/front", *SERVER_ORIGIN))
        .json(&msg)
        .send()
        .await?
        .json()
        .await?;
    Ok(res)
}

#[derive(Debug, Clone, Routable, PartialEq)]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/problem/:pid")]
    Problem { pid: Pid },
    #[route("/submit/:pid")]
    Submit { pid: Pid },
    #[route("/record/:rid")]
    Record { rid: Rid },
    #[route("/judge-status")]
    JudgeStatus {},
}

use judge_status::JudgeStatus;
use problem::Problem;
use record::Record;
use submit::Submit;

#[component]
// #[allow(non_snake_case)]
fn Home() -> Element {
    rsx! {
        h1 { "It's Mygoj !!! " }
    }
}

/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    rsx! {
        Outlet::<Route> {}
    }
}

#[component]
fn app() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

fn main() {
    dioxus::logger::init(Level::INFO).expect("logger init");
    std::panic::set_hook(Box::new(|info| {
        error!("Panic Occured\n{}", info);
        let _ = web_sys::window().unwrap().alert_with_message("panic");
    }));

    launch(app);
}
