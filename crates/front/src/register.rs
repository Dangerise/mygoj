use super::*;
use shared::user::*;

#[component]
pub fn UserRegister() -> Element {
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    let mut confirm = use_signal(String::new);
    let mut nickname = use_signal(String::new);
    let mut username = use_signal(String::new);

    let mut pwd_ne = use_signal(|| false);
    let mut completed = use_signal(|| false);

    let mut error_msg = use_signal(String::new);

    let register = move |_| {
        if password.cloned() != confirm.cloned() {
            pwd_ne.set(true);
            return;
        }
        pwd_ne.set(false);
        spawn(async move {
            let _: Uid = send_message(FrontMessage::RegisterUser(UserRegistration {
                username: username.cloned().into(),
                email: email.cloned().into(),
                password: password.cloned().into(),
                nickname: nickname.cloned().into(),
            }))
            .await
            .unwrap();
            completed.set(true);
        });
    };

    if completed.cloned() {
        spawn(async move {
            sleep(3000).await;
            let nav = navigator();
            nav.push(Route::Login {});
        });
        return rsx! {
            Common {
                content: concat!(
                    "Your account has been successfully registered ",
                    "We will jump to login page later",
                ),
            }
        };
    }

    use_effect(move || {
        if !shared::is_lowercase(&username()) {
            error_msg.set("username should be lowercase".into());
        }
    });

    rsx! {
        p { "nickname" }
        input {
            onchange: move |evt| {
                username.set(evt.value());
            },
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
        if pwd_ne.cloned() {
            p { "password not equal" }
        }
        p { "confirm" }
        input {
            onchange: move |evt| {
                confirm.set(evt.value());
            },
        }
        p { "nickname" }
        input {
            onchange: move |evt| {
                nickname.set(evt.value());
            },
        }
        {
            let err = error_msg.read();
            (!err.is_empty()).then(|| rsx! {
                div {
                    label { "error    " }
                    label { "{err}" }
                }
            })
        }
        button { onclick: register, "register" }
    }
}
