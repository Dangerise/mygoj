use super::*;
use shared::submission::*;

async fn submit_code(submission: &Submission) -> eyre::Result<Rid> {
    let url = format!("{}/api/submit", *SERVER_ORIGIN);
    let client = reqwest::Client::new();
    let rid: Rid = client
        .post(url)
        .json(submission)
        .send()
        .await?
        .json()
        .await?;
    Ok(rid)
}

#[component]
pub fn Submit(pid: Pid) -> Element {
    let mut code = use_signal(|| String::new());
    let rid: Signal<Option<Rid>> = use_signal(|| None);
    if let Some(rid) = *rid.read() {
        let nav = navigator();
        nav.push(Route::Record { rid });
    }
    rsx! {
        h1 { "submit to {pid}" }
        Link { to: Route::Problem { pid:pid.clone() }, "back to problem" }
        input {
            onchange: move |evt| {
                code.set(evt.value());
            },
        }
        button {
            onclick: move |_| {
                let submission = Submission {
                    code: code.cloned(),
                    pid: pid.clone(),
                };
                let mut rid = rid.clone();
                spawn(async move {
                    let submission = submission;
                    let t = submit_code(&submission).await.unwrap();
                    rid.set(Some(t));
                });
            },
            "submit"
        }
    }
}
