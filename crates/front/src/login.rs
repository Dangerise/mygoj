use super::*;

#[component]
pub fn Login() -> Element {
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());

    let login = move |_| {
        spawn(async move {
            let login_user: LoginedUser = send_message(FrontMessage::LoginUser(
                email.cloned().into(),
                password.cloned().into(),
            ))
            .await
            .unwrap();
            *LOGIN_STATE.write().unwrap() = Some(login_user);
            let nav = navigator();
            nav.push(Route::Home {});
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
