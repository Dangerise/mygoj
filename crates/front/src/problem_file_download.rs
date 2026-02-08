use super::*;
use shared::download::DownloadToken;
use web_sys::HtmlElement;
use web_sys::wasm_bindgen::JsCast;

#[component]
pub fn ProblemFileDownload(pid: Pid, path: String) -> Element {
    let pid = use_signal(|| pid);
    let path = use_signal(|| path);
    let token = use_resource(move || async move {
        let token: DownloadToken = send_message(FrontMessage::RequireProblemFileDownloadToken(
            pid(),
            path().into(),
        ))
        .await
        .unwrap();
        tracing::info!("download token is {}", token.encode());
        token
    });
    use_effect(move || {
        if token.read().is_some() {
            let document = gloo::utils::document();
            let a = document.get_element_by_id("download_link").unwrap();
            let a: HtmlElement = a.dyn_into().unwrap();
            a.click();
        }
    });
    rsx! {
        p { "wait a second" }
        if let Some(token) = *token.read() {
            {
                let url = format!(
                    "{}/api/front/problem_file_download/{}/{}?token={}",
                    *SERVER_URL,
                    pid,
                    path,
                    token.encode(),
                );
                rsx! {
                    a { id: "download_link", download: "{path}", href: url,
                        "if not triggered automatically, click me "
                    }
                }
            }
        }
    }
}
