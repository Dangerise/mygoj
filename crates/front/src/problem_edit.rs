use super::*;
use shared::problem::ProblemEditable;
use utility::loading_page;

#[component]
pub fn render_editable(mut editable: Signal<Option<ProblemEditable>>) -> Element {
    let mut time_limit = use_signal(|| editable.read().as_ref().unwrap().time_limit.to_string());
    let mut time_limit_error = use_signal(String::new);
    let mut memory_limit =
        use_signal(|| editable.read().as_ref().unwrap().memory_limit.to_string());
    let mut memory_limit_error = use_signal(String::new);
    use_effect(move || {
        let mut editable = editable.write();
        let editable = editable.as_mut().unwrap();
        let time_limit: u32 = match time_limit.read().parse() {
            Ok(v) => v,
            Err(_) => {
                time_limit_error.set("it should be a non-negative integer".into());
                return;
            }
        };
        if time_limit > 10_000 {
            time_limit_error.set("too large".into());
        }
        editable.time_limit = time_limit;
        let memory_limit: u32 = match memory_limit.read().parse() {
            Ok(v) => v,
            Err(_) => {
                memory_limit_error.set("it should be a non-negative integer".into());
                return;
            }
        };
        if memory_limit > (1 << 24) {
            memory_limit_error.set("too large".into());
            return;
        }
        editable.memory_limit = memory_limit;
    });
    rsx! {
        label { "title" }
        input {
            value: editable.read().as_ref().unwrap().title.clone(),
            onchange: move |evt| {
                editable.write().as_mut().unwrap().title = evt.value();
            },
        }
        label { "time_limit (ms)" }
        input {
            value: time_limit,
            onchange: move |evt| {
                time_limit.set(evt.value());
            },
        }
        {
            let msg = time_limit_error.read();
            if !msg.is_empty() {
                rsx! {
                    label { "{msg}" }
                }

            } else {
                rsx! {}
            }
        }
        label { "memory_limit (mb)" }
        input {
            value: memory_limit,
            onchange: move |evt| {
                memory_limit.set(evt.value());
            },
        }
        {
            let msg = memory_limit_error.read();
            if !msg.is_empty() {
                rsx! {
                    label { "{msg}" }
                }

            } else {
                rsx! {}
            }
        }
        label { "statement" }
        textarea {
            value: editable.read().as_ref().unwrap().statement.clone(),
            onchange: move |evt| {
                editable.write().as_mut().unwrap().title = evt.value();
            },
        }
    }
}

#[component]
pub fn ProblemEdit(pid: Pid) -> Element {
    let mut fetched = use_signal(|| false);
    let mut editable = use_signal(|| None);
    {
        let pid = pid.clone();
        use_future(move || {
            let pid = pid.clone();
            async move {
                let editable_val: ProblemEditable =
                    send_message(FrontMessage::GetProblemEditable(pid))
                        .await
                        .unwrap();
                editable.set(editable_val.into());
                fetched.set(true);
            }
        })
    };
    rsx! {
        {
            let pid = pid.0.as_str();
            rsx! {
                h2 { "Edit {pid}" }
                hr {}
            }
        }
        {
            if fetched.cloned() {
                rsx! {

            
                    render_editable { editable }
                    button { onclick: move |_| {}, "save" }
                }
            } else {
                rsx! {
                    loading_page {}
                }
            }
        }
    }
}
