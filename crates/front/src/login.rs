use super::*;
use std::sync::Mutex;

static REDIRECT: Mutex<Option<Route>> = Mutex::new(None);
static TIP: Mutex<bool> = Mutex::new(false);

pub fn login_required(redirect: Route) {
    let mut red = REDIRECT.lock().unwrap();
    *red = Some(redirect);
    *TIP.lock().unwrap() = true;
    navigator().push(Route::Login {});
}

async fn login(email: String, password: String) {
    let (token, login_user): (String, LoginedUser) =
        send_message(FrontMessage::LoginUser(email.into(), password.into()))
            .await
            .unwrap();
    storage()
        .set(shared::constant::LOGIN_TOKEN, &token)
        .unwrap();
    *LOGIN_STATE.write() = Some(login_user);
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
        {
            let mut tip = TIP.lock().unwrap();
            if *tip {
                *tip = false;
                rsx! {
                    p { "you need to login to access the page" }
                }
            } else {
                rsx! {}
            }
        }
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
        button { onclick: login, "login" }
    }
}
