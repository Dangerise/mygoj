use super::*;
use shared::submission::*;

#[component]
pub fn Submit(pid: Pid) -> Element {
    if LOGIN_STATE.read().is_none() {
        login::login_required(Route::Submit { pid: pid.clone() });
    }
    let mut code = use_signal(String::new);
    rsx! {
        h1 { "submit to {pid}" }
        Link { to: Route::Problem { pid: pid.clone() }, "back to problem" }
        textarea {
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
                spawn(async move {
                    let submission = submission;
                    let rid: Rid = send_message(FrontMessage::Submit(submission)).await.unwrap();
                    navigator().push(Route::Record { rid });
                });
            },
            "submit"
        }
    }
}
