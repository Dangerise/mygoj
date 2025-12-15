use super::*;

#[component]
pub fn Logout() -> Element {
    let mut done = use_signal(|| false);
    use_future(move || async move {
        let _: () = send_message(FrontMessage::Logout).await.unwrap();
        done.set(true);
        sleep(1500).await;
        navigator().push(Route::Home {});
    });
    if done.cloned() {
        rsx! {
            p { "you have been successfully logout" }
        }
    } else {
        rsx! {
            p { "please wait a moment" }
        }
    }
}
