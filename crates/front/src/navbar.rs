use super::*;

#[component]
pub fn Navbar() -> Element {
    rsx! {
        h1 { "MyGoJ" }
        Link { to: Route::Home {}, "Home " }
        Link { to: Route::ProblemPage { index: 0 }, "Problems " }
        {
            if let Some(login_state) = &*LOGIN_STATE.read() {
                let nickname = &login_state.nickname;
                rsx! {
                    p { "{nickname}" }
                    Link { to: Route::Logout {}, "Logout " }
                }
            } else {
                rsx! {
                    Link { to: Route::Login {}, "Login " }
                    Link { to: Route::UserRegister {}, "Register " }
                }
            }
        }
        hr {}
        Outlet::<Route> {
        }
    }
}
