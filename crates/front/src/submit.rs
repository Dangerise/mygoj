use super::*;
use shared::submission::*;

#[component]
pub fn Submit(pid: Pid) -> Element {
    let mut code = use_signal(String::new);
    let rid: Signal<Option<Rid>> = use_signal(|| None);
    if let Some(rid) = *rid.read() {
        let nav = navigator();
        nav.push(Route::Record { rid });
    }
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
                let mut rid = rid;
                spawn(async move {
                    let submission = submission;
                    let t: Rid = send_message(FrontMessage::Submit(submission)).await.unwrap();
                    rid.set(Some(t));
                });
            },
            "submit"
        }
    }
}
