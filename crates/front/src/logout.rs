use super::*;

pub fn clear_cache() {
    storage()
        .remove_item(shared::constant::LOGIN_TOKEN)
        .unwrap();
    *LOGIN_STATE.write() = None;
}

#[component]
pub fn Logout() -> Element {
    clear_cache();
    let mut done = use_signal(|| false);
    use_future(move || async move {
        let _: () = send_message(FrontMessage::Logout).await.unwrap();
        done.set(true);
        sleep(1500).await;
        navigator().push(Route::Home {});
    });
    rsx! {
        Common { content: if done.cloned() { "you have been successfully logout" } else { "please wait a moment" } }
    }
}
