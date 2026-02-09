use super::*;
pub use shared::problem::ProblemFront;

#[component]
fn loading_page() -> Element {
    rsx! {
        p { "Loading, please wait`" }
    }
}

#[component]
fn wrong_page() -> Element {
    rsx! {
        p { "Something goes wrong " }
    }
}

#[component]
fn render_problem(front: ProblemFront) -> Element {
    tracing::info!("render problem {:?}", &front);
    let login = LOGIN_STATE.read();
    let login = &*login;
    let edit = if let Some(login) = login {
        front.can_be_edited_by(login)
    } else {
        false
    };
    let ProblemFront {
        pid,
        title,
        statement,
        time_limit,
        memory_limit,
        ..
    } = front;
    rsx! {
        Link {
            to: Route::ProblemFileList {
                pid: pid.clone(),
            },
            "files"
        }
        {"   "}
        if edit {
            Link {
                to: Route::ProblemEdit {
                    pid: pid.clone(),
                },
                "edit"
            }
        }
        h1 { "{pid} {title}" }
        p { "time {time_limit} ms memory {memory_limit} mb" }
        Markdown { content: statement }
        Link { to: Route::Submit { pid }, "To submit" }
    }
}

#[component]
pub fn Problem(pid: Pid) -> Element {
    let front = {
        let pid = pid.clone();
        use_resource(move || {
            let pid = pid.clone();
            async move { send_message::<ProblemFront>(FrontMessage::GetProblemFront(pid)).await }
        })
    };
    if let Some(front) = &*front.read() {
        match front {
            Ok(front) => rsx! {
                render_problem { front: front.clone() }
            },
            Err(err) => {
                tracing::error!("error {:?}", err);
                rsx! {
                    wrong_page {}
                }
            }
        }
    } else {
        rsx! {
            loading_page {}
        }
    }
}
