use super::*;
use shared::problem::ProblemFile;

#[component]
fn ViewContent(pid: Pid, path: String, meta: ProblemFile) -> Element {
    let name = path.rsplit("/").next().unwrap().to_string();
    let pid = use_signal(|| pid);
    let path = use_signal(|| path);
    let bytes = use_resource(move || async move {
        let bytes = download_problem_file(&pid(), &path()).await.unwrap();
        bytes
    });
    rsx! {
        if let Some(bytes) = bytes.read().cloned() {
            FileView { name, bytes }
        } else {
            "loading"
        }
    }
}

#[component]
pub fn ProblemFileView(pid: Pid, path: String) -> Element {
    let pid = use_signal(|| pid);
    let path = use_signal(|| path);
    let meta = use_resource(move || async move {
        let g: ProblemFile = send_message(FrontMessage::GetProblemFileMeta(
            pid(),
            path.read().as_str().into(),
        ))
        .await
        .unwrap();
        g
    });
    rsx! {
        button {
            onclick: move |_| {
                web_sys::window()
                    .unwrap()
                    .open_with_url(&format!("/problem/{}/file_download/{}", pid(), path()))
                    .unwrap();
            },
            "download"
        }
        if let Some(meta) = meta.read().cloned() {
            ViewContent { pid: pid(), path: path(), meta }
        }
    }
}
