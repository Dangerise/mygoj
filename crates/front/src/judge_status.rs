use super::*;
use shared::judge::JudgeSignal;

async fn get_judge_signals() -> eyre::Result<Vec<JudgeSignal>> {
    let url = format!("{}/api/judge_machines", *SERVER_ORIGIN);
    let signals: Vec<JudgeSignal> = reqwest::get(url).await?.json().await?;
    Ok(signals)
}

#[component]
fn display_single(sig: JudgeSignal) -> Element {
    let JudgeSignal {
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
        hr {  }
    }
}

#[component]
fn display_signals(judge_signals: Vec<JudgeSignal>) -> Element {
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
    let mut judge_signals_res = use_resource(|| async { get_judge_signals().await });
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
