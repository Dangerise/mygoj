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

async fn login(email: String, password: String) -> eyre::Result<()> {
    let resp = reqwest::Client::new()
        .post(format!("{}/api/front/login", *SERVER_URL))
        .basic_auth(email, Some(password))
        .send()
        .await?;
    if resp.status() != StatusCode::OK {
        let err: ServerError = resp.json().await?;
        return Err(err.into());
    }
    let (token, login_user): (String, LoginedUser) = resp.json().await?;
    storage()
        .set_item(shared::constant::LOGIN_TOKEN, &token)
        .unwrap();
    *LOGIN_STATE.write() = Some(login_user);
    Ok(())
}

#[component]
pub fn Login() -> Element {
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);

    let mut error_msg = use_signal(|| None);
    let login = move |_| {
        spawn(async move {
            match login(email.cloned(), password.cloned()).await {
                Ok(_) => {}
                Err(err) => {
                    let err = err.split();
                    match err {
                        ErrorKind::Client(err) => {
                            error_msg.set(format!("client error \n{err:#?}").into());
                        }
                        ErrorKind::Server(err) => match &err {
                            ServerError::UserNotFound => {
                                error_msg.set("user not found".to_string().into());
                            }
                            ServerError::PasswordWrong => {
                                error_msg.set("wrong password".to_string().into());
                            }
                            _ => {
                                error_msg.set(format!("other error \n{err:#?}").into());
                            }
                        },
                    }
                    return;
                }
            }
            let nav = navigator();
            let redirect = REDIRECT.lock().unwrap();
            if let Some(redirect) = &*redirect {
                nav.push(redirect.clone());
            } else {
                nav.push(Route::Home {});
            }
        });
    };

    let dialog = || -> Element {
        let msg = error_msg
            .read()
            .as_ref()
            .unwrap_or(&String::new())
            .to_string();
        rsx! {
            DialogRoot { open: error_msg.read().is_some(),
                DialogContent {
                    DialogTitle { "Error" }
                    DialogDescription {
                        Multilines { content: msg }
                    }
                    button {
                        onclick: move |_| {
                            error_msg.set(None);
                        },
                        "confirm"
                    }
                }
            }
        }
    };

    rsx! {
        {dialog()}
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
