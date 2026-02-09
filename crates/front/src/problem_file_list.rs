use super::*;
use shared::problem::ProblemFile;

#[component]
pub fn ProblemFileList(pid: Pid) -> Element {
    let pid = use_signal(|| pid);
    let files = use_resource(move || async move {
        let files: Vec<ProblemFile> = send_message(FrontMessage::GetProblemFiles(pid()))
            .await
            .unwrap();
        files
    });
    rsx! {
        if let Some(files) = &*files.read() {
            for f in files {
                {
                    let path = f.path.clone();
                    rsx! {
                        p { "{path}" }
                        button {
                            onclick: {
                                let path = path.clone();
                                move |_| {
                                    web_sys::window()
                                        .unwrap()
                                        .open_with_url(&format!("/problem/{}/file_view/{}", pid(), &path))
                                        .unwrap();
                                }
                            },
                            "view"
                        }
                        button {
                            onclick: {
                                let path = path.clone();
                                move |_| {
                                    web_sys::window()
                                        .unwrap()
                                        .open_with_url(&format!("/problem/{}/file_download/{}", pid(), &path))
                                        .unwrap();
                                }
                            },
                            "download"
                        }
                    }
                }
            }
        }
    }
}
