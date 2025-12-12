use super::*;

mod inner {
    use super::SERVER_ORIGIN;
    use shared::judge::{AllJudgeResult, SingleJudgeResult};
    use shared::record::*;
    async fn get_record(rid: Rid) -> eyre::Result<Record> {
        let url = format!("{}/api/record?rid={}", *SERVER_ORIGIN, rid);
        let record = reqwest::get(url).await?.json().await?;
        Ok(record)
    }

    use dioxus::prelude::*;

    #[component]
    fn show_all_results(status: Vec<Option<SingleJudgeResult>>) -> Element {
        rsx! {
            for (idx,case) in status.into_iter().enumerate() {
                if let Some(SingleJudgeResult { verdict, memory_used, time_used }) = case {
                    p { "#{idx} {verdict} {time_used} ms {memory_used} mb" }
                } else {
                    p { "#{idx} Running" }
                }
            }
        }
    }

    #[component]
    fn show_record_status(status: RecordStatus) -> Element {
        match status {
            RecordStatus::Waiting => {
                rsx! {
                    p { "Waiting" }
                }
            }
            RecordStatus::Running(status) => {
                rsx! {
                    p { "Running" }
                    show_all_results { status }
                }
            }
            RecordStatus::Completed(status) => {
                let AllJudgeResult {
                    cases,
                    verdict,
                    memory_used,
                    max_time,
                    sum_time,
                } = status;
                let verdict = format!("{}", verdict);
                let status = cases.into_iter().map(|x| Some(x)).collect();
                rsx! {
                    p { "max time {max_time} sum time {sum_time} memory used {memory_used} " }
                    p { "{verdict}" }
                    show_all_results { status }
                }
            }
            RecordStatus::CompileError(err) => {
                let err = format!("{}", err);
                rsx! {
                    p { "Compile Error" }
                    textarea { "{err}" }
                }
            }
            RecordStatus::Compiling => {
                rsx! {
                    p { "Compiling" }
                }
            }
        }
    }

    #[component]
    pub fn record_page(rid: Rid) -> Element {
        let record = use_resource(move || async move { get_record(rid).await });
        if let Some(record) = &*record.read() {
            let record = record.as_ref().unwrap();
            let Record {
                rid: _,
                pid,
                code,
                status,
                ..
            } = record;
            rsx! {
                p { "Problem {pid}" }
                show_record_status { status: status.clone() }
                textarea { "{code}" }
            }
        } else {
            rsx! {
                p { "Loading" }
            }
        }
    }
}
use inner::record_page;

#[component]
pub fn Record(rid: Rid) -> Element {
    rsx! {
        record_page { rid }
    }
}
