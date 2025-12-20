use dioxus::logger::tracing::{self, Level};
use dioxus::prelude::*;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use shared::error::ServerError;
use shared::front::FrontMessage;
use shared::problem::Pid;
use shared::record::Rid;
use shared::user::LoginedUser;
use std::sync::LazyLock;

mod error;
mod home;
mod judge_status;
mod login;
mod login_outdated;
mod logout;
mod md;
mod navbar;
mod notfound;
mod problem;
mod record;
mod register;
mod submit;
mod utility;
mod components;

use error::{ErrorKind, Split as _};
use md::Markdown;
use utility::*;
use components::*;

static LOGIN_STATE: GlobalSignal<Option<LoginedUser>> = GlobalSignal::new(|| None);
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

use home::Home;
use judge_status::JudgeStatus;
use login::Login;
use login_outdated::LoginOutDated;
use logout::Logout;
use navbar::Navbar;
use notfound::NotFound;
use problem::Problem;
use record::Record;
use register::UserRegister;
use submit::Submit;

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
