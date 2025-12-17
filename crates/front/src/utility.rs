use super::*;

#[track_caller]
pub fn storage() -> web_sys::Storage {
    web_sys::window().unwrap().local_storage().unwrap().unwrap()
}

pub fn ws_origin() -> String {
    let origin = SERVER_URL.as_str();
    assert!(origin.starts_with("http") || origin.starts_with("https"));
    format!(
        "ws{}",
        if origin.starts_with("http") {
            &origin["http".len()..]
        } else if origin.starts_with("https") {
            &origin["https".len()..]
        } else {
            unreachable!()
        }
    )
}

pub async fn sleep(ms: u32) {
    let js = include_str!("sleep.js");
    let js = String::from(js).replace("TIME", &format!("{ms}"));
    dioxus::document::eval(&js).await.unwrap();
}

pub fn auto_error(err: ServerError) {
    tracing::error!("{err:#?}");
    match err {
        ServerError::LoginOutDated => {
            login_outdated::login_outdated();
        }
        ServerError::NotFound => {
            let url = web_sys::window().unwrap().location().as_string().unwrap();
            notfound::notfound(url);
        }
        _ => {}
    }
}

pub async fn send_message<T>(msg: FrontMessage) -> eyre::Result<T>
where
    T: DeserializeOwned,
{
    let storage = storage();
    let token = storage.get(shared::constant::LOGIN_TOKEN).unwrap();

    let mut req = reqwest::Client::new()
        .post(format!("{}/api/front", *SERVER_URL))
        .json(&msg);
    if let Some(token) = token {
        req = req.header(shared::constant::LOGIN_TOKEN, token);
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
                Err::<(), _>(err).unwrap();
            }
            None
        }
    };
    *LOGIN_STATE.write().unwrap() = logined_user;
}
