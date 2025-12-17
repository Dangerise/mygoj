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
mod utility;

use utility::*;

static LOGIN_STATE: RwLock<Option<LoginedUser>> = RwLock::new(None);

static SERVER_URL: LazyLock<String> = LazyLock::new(|| {
    #[cfg(not(feature = "independent"))]
    {
        web_sys::window().unwrap().origin()
    }
    #[cfg(feature = "independent")]
    {
        std::env::var("SERVER_URL").unwrap_or("http://localhost:5800".to_string())
    }
});

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

#[component]
fn Navbar() -> Element {
    rsx! {
        Outlet::<Route> {}
    }
}

#[component]
fn app() -> Element {
    let mut start = use_signal(|| false);
    use_future(move || async move {
        init_login_state().await;
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
