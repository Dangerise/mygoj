use super::*;

#[component]
pub fn Home() -> Element {
    let login_state = LOGIN_STATE.read().clone();

    let welcome = || {
        if let Some(login_state) = login_state {
            let nickname = login_state.nickname;
            rsx! { "Welcome ! {nickname}" }
        } else {
            rsx! { "Welcome ! but please login first !" }
        }
    };

    rsx! {
        div { class: "home",
            Markdown { md: "# It's Mygoj !!!" }
        }
        {welcome()}
    }
}