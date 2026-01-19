use super::*;

pub fn clear_cache() {
    storage()
        .remove_item(shared::constant::LOGIN_TOKEN)
        .unwrap();
    *LOGIN_STATE.write() = None;
}

async fn logout() -> eyre::Result<()> {
    let storage = storage();
    let Some(token) = storage.get(shared::constant::LOGIN_TOKEN).unwrap() else {
        return Ok(());
    };

    let resp = reqwest::Client::new()
        .post(format!("{}/api/front/logout", *SERVER_URL))
        .bearer_auth(token)
        .send()
        .await?;

    storage.remove_item(shared::constant::LOGIN_TOKEN).unwrap();

    if resp.status() != StatusCode::OK {
        let err: ServerError = resp.json().await?;
        return Err(err.into());
    }
    Ok(())
}

#[component]
pub fn Logout() -> Element {
    clear_cache();
    let mut done = use_signal(|| false);
    use_future(move || async move {
        logout().await.unwrap();
        done.set(true);
        sleep(1500).await;
        navigator().push(Route::Home {});
    });
    rsx! {
        Common { content: if done.cloned() { "you have been successfully logout" } else { "please wait a moment" } }
    }
}
