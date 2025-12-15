use super::*;

pub fn login_outdated() {
    *LOGIN_STATE.write().unwrap() = None;
    navigator().push(Route::LoginOutDated {});
}

#[component]
pub fn LoginOutDated() -> Element {
    use_future(|| async {
        sleep(2000).await;
        navigator().push(Route::Login {});
    });
    rsx! {
        p { "Your login has out dated !" }
        p { "please login again " }
        p { "We will jump to login page later !" }
    }
}
