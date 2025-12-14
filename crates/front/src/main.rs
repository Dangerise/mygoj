use dioxus::logger::tracing::{self, Level};
use dioxus::prelude::*;
use serde::de::DeserializeOwned;
use shared::front::FrontMessage;
use shared::problem::Pid;
use shared::record::Rid;
use shared::user::LoginedUser;
use std::sync::{LazyLock, RwLock};

mod judge_status;
mod login;
mod problem;
mod record;
mod register;
mod submit;

static LOGIN_STATE: RwLock<Option<LoginedUser>> = RwLock::new(None);

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

async fn sleep(ms: u32) {
    let js = include_str!("sleep.js");
    let js = String::from(js).replace("TIME", &format!("{ms}"));
    dioxus::document::eval(&js).await.unwrap();
}

async fn send_message<T>(msg: FrontMessage) -> eyre::Result<T>
where
    T: DeserializeOwned,
{
    let mut req = reqwest::Client::new()
        .post(format!("{}/api/front", *SERVER_ORIGIN))
        .json(&msg);
    if let Some(login_state) = &*LOGIN_STATE.read().unwrap() {
        req = req.header(shared::headers::LOGIN_STATE, login_state.token.encode());
    }
    let res = req.send().await?.json().await?;
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
    #[route("/login")]
    Login {},
    #[route("/register")]
    UserRegister {},
}

use judge_status::JudgeStatus;
use login::Login;
use problem::Problem;
use record::Record;
use register::UserRegister;
use submit::Submit;

#[component]
fn Home() -> Element {
    let login_state = LOGIN_STATE.read().unwrap().clone();

    let welcome = || {
        if let Some(login_state) = login_state {
            let nickname = login_state.nickname;
            rsx! {
                "Welcome ! {nickname}"
            }
        } else {
            rsx! {
                "Welcome ! but please login first !"
            }
        }
    };

    rsx! {
        h1 { "Hello, It's Mygoj !!! " }
        {
            welcome()
        }
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
    }));

    launch(app);
}
