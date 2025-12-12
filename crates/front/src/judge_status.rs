use super::*;
use shared::judge::JudgeMachineSignal;

#[component]
fn display_single(sig: JudgeMachineSignal) -> Element {
    let JudgeMachineSignal {
        cpu_usage,
        cpu_name,
        tasks,
        uuid,
        timestamp: _,
        hostname,
        system_name,
    } = sig;
    let len = tasks.len();
    let system_name = system_name.as_deref().unwrap_or("null");
    let hostname = hostname.as_deref().unwrap_or("null");
    rsx! {
        p { "uuid {uuid}" }
        p { "CPU {cpu_name}" }
        p { "{system_name}" }
        p { "{hostname}" }
        p { "CPU usage {cpu_usage}%" }
        p { "tasks {len}" }
        hr {}
    }
}

#[component]
fn display_signals(judge_signals: Vec<JudgeMachineSignal>) -> Element {
    if judge_signals.is_empty() {
        rsx! {
            p { "no judge machine connected yet" }
        }
    } else {
        rsx! {
            for item in &judge_signals {
                display_single { sig: item.clone() }
            }
        }
    }
}

#[component]
pub fn JudgeStatus() -> Element {
    let mut judge_signals_res = use_resource(|| async {
        send_message::<Vec<JudgeMachineSignal>>(FrontMessage::CheckJudgeMachines).await
    });
    if let Some(judge_signals) = &*judge_signals_res.read() {
        let judge_signals = judge_signals.as_ref().unwrap();
        rsx! {
            button {
                onclick: move |_| {
                    judge_signals_res.restart();
                },
                "refresh"
            }
            display_signals { judge_signals: judge_signals.clone() }
        }
    } else {
        rsx! {
            p { "loading" }
        }
    }
}
