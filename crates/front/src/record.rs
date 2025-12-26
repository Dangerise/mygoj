use super::*;

mod inner {
    use super::send_message;
    use crate::ws_origin;
    use dioxus::logger::tracing;
    use dioxus::prelude::*;
    use futures_util::StreamExt;
    use shared::front::FrontMessage;
    use shared::judge::{AllJudgeResult, SingleJudgeResult};
    use shared::record::*;
    use ws_stream_wasm::*;

    #[component]
    fn show_all_results(status: Vec<Option<SingleJudgeResult>>) -> Element {
        rsx! {
            for (idx , case) in status.into_iter().enumerate() {
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

    fn handle_record_message(msg: RecordMessage, mut record: Signal<Option<Record>>) -> bool {
        let mut wr = record.write();
        let status = &mut wr.as_mut().unwrap().status;
        match msg {
            RecordMessage::CompileError(ce) => {
                *status = RecordStatus::CompileError(ce);
                return false;
            }
            RecordMessage::Compiled(cnt) => {
                *status = RecordStatus::Running(vec![const { None }; cnt])
            }
            RecordMessage::NewSingleResult(idx, single) => {
                let RecordStatus::Running(status) = status else {
                    unreachable!()
                };
                status[idx] = Some(single);
            }
            RecordMessage::Completed(all) => {
                *status = RecordStatus::Completed(all);
                return false;
            }
            RecordMessage::Compiling => {
                *status = RecordStatus::Compiling;
            }
        }
        true
    }

    async fn ws(rid: Rid, record: Signal<Option<Record>>) {
        let url = format!("{}/api/front/record_ws?rid={}", ws_origin(), rid);
        tracing::info!("try to ws to {url}");
        let Ok((_, mut stream)) = WsMeta::connect(&url, None).await else {
            tracing::error!("fail to establish websocket connection for rid {rid}");
            return;
        };
        tracing::info!("ws on {url}");
        loop {
            let Some(msg) = stream.next().await else {
                tracing::info!("no event so close for rid {rid}");
                return;
            };
            let WsMessage::Text(text) = msg else {
                tracing::error!("not text rid {rid}");
                return;
            };
            let msg: RecordMessage = serde_json::from_str(&text).unwrap();
            tracing::info!("recv msg {msg:#?}");
            if !handle_record_message(msg, record) {
                break;
            }
        }
    }

    async fn manual_refresh(rid: Rid, mut record: Signal<Option<Record>>) {
        loop {
            let Ok(t): Result<Record, _> = send_message(FrontMessage::GetRecord(rid)).await else {
                return;
            };
            tracing::info!("udpate record {t:#?}");
            let stop = t.status.done();
            record.set(Some(t));
            if stop {
                return;
            }
        }
    }

    #[component]
    pub fn record_page(rid: Rid) -> Element {
        let mut record = use_signal(|| None);
        use_future(move || async move {
            let t: Record = send_message(FrontMessage::GetRecord(rid)).await.unwrap();
            let done = t.status.done();
            record.set(Some(t));
            if !done {
                #[cfg(feature = "ws_for_record")]
                ws(rid, record.clone()).await;
                #[cfg(not(feature = "ws_for_record"))]
                manual_refresh(rid, record).await;
            }
            ()
        });
        if let Some(record) = &*record.read() {
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
