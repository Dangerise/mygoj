use super::*;

#[component]
pub fn Navbar() -> Element {
    rsx! {
        h1 { "MyGoJ" }
        Link { to: Route::Home {}, "Home " }
        if LOGIN_STATE.read().unwrap().is_none() {
            Link { to: Route::Login {}, "Login " }
            Link { to: Route::UserRegister {}, "Register " }
        }
        Outlet::<Route> {
        }
    }
}
