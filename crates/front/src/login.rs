use super::*;
use std::sync::Mutex;

static REDIRECT: Mutex<Option<Route>> = Mutex::new(None);

fn login_required(redirect: Route) -> Element {
    rsx! {
        p {
            "you need to login to access the page"
        }
        Login {  }
    }
}

async fn login(email: String, password: String) {
    let (token, login_user): (String, LoginedUser) =
        send_message(FrontMessage::LoginUser(email.into(), password.into()))
            .await
            .unwrap();
    storage()
        .set(shared::constant::LOGIN_TOKEN, &token)
        .unwrap();
    *LOGIN_STATE.write().unwrap() = Some(login_user);
}

#[component]
pub fn Login() -> Element {
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());

    let login = move |_| {
        spawn(async move {
            login(email.cloned(), password.cloned()).await;
            let nav = navigator();
            let redirect = REDIRECT.lock().unwrap();
            if let Some(redirect) = &*redirect {
                nav.push(redirect.clone());
            } else {
                nav.push(Route::Home {});
            }
        });
    };

    rsx! {
        p { "email" }
        input {
            onchange: move |evt| {
                email.set(evt.value());
            },
        }
        p { "password" }
        input {
            onchange: move |evt| {
                password.set(evt.value());
            },
        }
        button { onclick: login ,"login"}
    }
}
