use super::*;

pub fn now() -> i64 {
    (web_sys::js_sys::Date::now() / 1000.) as i64
}

#[track_caller]
pub fn storage() -> web_sys::Storage {
    web_sys::window().unwrap().local_storage().unwrap().unwrap()
}

#[track_caller]
pub fn login_token() -> Option<String> {
    storage().get(shared::constant::LOGIN_TOKEN).unwrap()
}

#[track_caller]
pub fn remove_login_token() {
    storage()
        .remove_item(shared::constant::LOGIN_TOKEN)
        .unwrap()
}

pub fn ws_origin() -> String {
    let origin = SERVER_URL.as_str();
    assert!(origin.starts_with("http") || origin.starts_with("https"));
    format!(
        "ws{}",
        if let Some(s) = origin.strip_prefix("http") {
            s
        } else if let Some(s) = origin.strip_prefix("https") {
            s
        } else {
            unreachable!()
        }
    )
}

#[component]
pub fn Common(content: String) -> Element {
    rsx! {
        div { class: "common",
            Markdown { md: content }
        }
    }
}

pub async fn sleep(ms: u32) {
    let js = include_str!("sleep.js");
    let js = String::from(js).replace("TIME", &format!("{ms}"));
    dioxus::document::eval(&js).await.unwrap();
}

// pub fn auto_error(err: ServerError) {
//     tracing::error!("{err:#?}");
//     match err {
//         ServerError::LoginOutDated => {
//             login_outdated::login_outdated();
//         }
//         ServerError::NotFound => {
//             let url = web_sys::window().unwrap().location().as_string().unwrap();
//             notfound::notfound(url);
//         }
//         _ => {}
//     }
// }

pub async fn send_message<T>(msg: FrontMessage) -> eyre::Result<T>
where
    T: DeserializeOwned,
{
    let mut req = reqwest::Client::new()
        .post(format!("{}/api/front", *SERVER_URL))
        .json(&msg);
    if let Some(token) = login_token() {
        req = req.bearer_auth(token);
    }
    let resp = req.send().await?;
    if resp.status() != StatusCode::OK {
        let err: ServerError = resp.json().await?;
        return Err(err.into());
    }
    let res = resp.json().await?;
    Ok(res)
}

pub async fn init_login_state() {
    let logined_user: Option<LoginedUser> = match send_message(FrontMessage::GetLoginedUser).await {
        Ok(ret) => ret,
        Err(err) => {
            if let Some(ServerError::LoginOutDated) = err.downcast_ref::<ServerError>() {
                tracing::info!("login outdated");
                logout::clear_cache();
            } else {
                panic!("{err:#?}")
            }
            None
        }
    };
    tracing::info!("login user {logined_user:#?}");
    *LOGIN_STATE.write() = logined_user;
}

#[component]
pub fn loading_page() -> Element {
    rsx! { "Loading" }
}

pub fn time_diff(diff: u64) -> String {
    const MIN: u64 = 60;
    const HOUR: u64 = MIN * 60;
    const DAY: u64 = HOUR * 24;
    const MONTH: u64 = DAY * 30;
    const YEAR: u64 = MONTH * 12;
    if diff > YEAR {
        format!("{} year ago", diff / YEAR)
    } else if diff > MONTH {
        format!("{} month ago", diff / MONTH)
    } else if diff > DAY {
        format!("{} day ago", diff / DAY)
    } else if diff > HOUR {
        format!("{} hour ago", diff / HOUR)
    } else if diff > MIN {
        format!("{} minute ago", diff / MIN)
    } else {
        "just now".to_string()
    }
}

#[component]
pub fn LoadingDialog(loading: bool) -> Element {
    rsx! {
        document::Stylesheet { href: asset!("assets/loading-dialog.css") }
        DialogRoot { open: loading,
            DialogContent {
                div { class: "loading-spinner" }
            }
        }
    }
}
