use dioxus::logger::tracing::{self, Level};
use dioxus::prelude::*;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use shared::error::ServerError;
use shared::front::FrontMessage;
use shared::problem::Pid;
use shared::record::Rid;
use shared::user::LoginedUser;
use std::sync::{LazyLock, RwLock};

mod judge_status;
mod login;
mod login_outdated;
mod logout;
mod notfound;
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

fn ws_origin() -> String {
    let origin = SERVER_ORIGIN.as_str();
    assert!(origin.starts_with("http") || origin.starts_with("https"));
    format!(
        "ws{}",
        if origin.starts_with("http") {
            &origin["http".len()..]
        } else if origin.starts_with("https") {
            &origin["https".len()..]
        } else {
            unreachable!()
        }
    )
}

async fn sleep(ms: u32) {
    let js = include_str!("sleep.js");
    let js = String::from(js).replace("TIME", &format!("{ms}"));
    dioxus::document::eval(&js).await.unwrap();
}

async fn send_message<T>(msg: FrontMessage) -> eyre::Result<T>
where
    T: DeserializeOwned,
{
    let req = reqwest::Client::new()
        .post(format!("{}/api/front", *SERVER_ORIGIN))
        .json(&msg);
    let resp = req.send().await?;
    if resp.status() != StatusCode::OK {
        let err: ServerError = resp.json().await?;
        return Err(err.into());
    }
    let res = resp.json().await?;
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
    #[route("/judge_status")]
    JudgeStatus {},
    #[route("/login")]
    Login {},
    #[route("/register")]
    UserRegister {},
    #[route("/login_outdated")]
    LoginOutDated {},
    #[route("/notfound")]
    NotFound {},
    #[route("/logout")]
    Logout {},
}

use judge_status::JudgeStatus;
use login::Login;
use login_outdated::LoginOutDated;
use logout::Logout;
use notfound::NotFound;
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

fn handle_server_error(err: ServerError) {
    tracing::error!("{err:#?}");
    match err {
        ServerError::LoginOutDated => {
            login_outdated::login_outdated();
        }
        ServerError::NotFound => {
            let url = web_sys::window().unwrap().location().as_string().unwrap();
            notfound::notfound(url);
        }
        _ => {}
    }
}

#[component]
fn app() -> Element {
    let mut start = use_signal(|| false);
    use_future(move || async move {
        let logined_user: Option<LoginedUser> =
            match send_message(FrontMessage::GetLoginedUser).await {
                Ok(ret) => ret,
                Err(err) => {
                    if let Some(err) = err.downcast_ref::<ServerError>() {
                        handle_server_error(err.clone());
                    } else {
                        Err::<(), _>(err).unwrap();
                    }
                    None
                }
            };
        *LOGIN_STATE.write().unwrap() = logined_user;
        start.set(true);
    });

    if start.cloned() {
        rsx! {
            Router::<Route> {}
        }
    } else {
        rsx!()
    }
}

fn main() {
    dioxus::logger::init(Level::INFO).expect("logger init");
    std::panic::set_hook(Box::new(|info| {
        error!("Panic Occured\n{}", info);
    }));

    launch(app);
}
