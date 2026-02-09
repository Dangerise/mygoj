use super::*;

// pub fn login_outdated() {
//     logout::clear_cache();
//     navigator().push(Route::LoginOutDated {});
// }

#[component]
pub fn LoginOutDated() -> Element {
    use_future(|| async {
        sleep(2000).await;
        navigator().push(Route::Login {});
    });
    rsx! {
        Common { content: r#"Your login has out dataed !\nplease login againWe will jump to login page later"# }
    }
}
