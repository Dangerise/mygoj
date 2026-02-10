use super::*;
use shared::problem::*;

#[component]
pub fn Inner(list: Vec<ProblemProfile>, index: u64, count: u64) -> Element {
    let mut jump_to = use_signal(|| index);
    rsx! {
        for profile in list {
            p {  
                Link {
                    to: Route::Problem {
                        pid: profile.pid.clone(),
                    },
                    {format!("{} {}", profile.pid, profile.title)}
                }
            }
        }
        div {
            if index > 0 {
                Link {
                    to: Route::ProblemPage {
                        index: index - 1,
                    },
                    "<"
                }
            } else {
                a { "<" }
            }
            input {
                value: jump_to(),
                onchange: move |evt| {
                    if let Ok(to) = evt.parsed() {
                        jump_to.set(to);
                    }
                },
            }
            label { "/{count}" }
            if index + 1 < count {
                Link {
                    to: Route::ProblemPage {
                        index: index + 1,
                    },
                    ">"
                }
            } else {
                a { ">" }
            }
        }
    }
}

#[component]
pub fn ProblemPage(index: u64) -> Element {
    let count = use_resource(|| async move {
        let count: u64 = send_message(FrontMessage::GetProblemsPageCount)
            .await
            .unwrap();
        count
    });
    let list = use_resource(move || async move {
        let list: Vec<ProblemProfile> = send_message(FrontMessage::GetProblemsPage(index))
            .await
            .unwrap();
        list
    });
    rsx! {
        if let Some(count) = count() && let Some(list) = list() {
            Inner { list, index, count }
        }
    }
}
