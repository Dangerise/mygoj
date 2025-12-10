use super::*;

mod inner {
    use super::SERVER_ORIGIN;
    use shared::record::Record;
    async fn get_record(rid: u64) -> eyre::Result<Record> {
        let url = format!("{}/api/record?rid={}", *SERVER_ORIGIN, rid);
        let record = reqwest::get(url).await?.json().await?;
        Ok(record)
    }

    use dioxus::prelude::*;
    #[component]
    pub fn record_page(rid: u64) -> Element {
        let record = use_resource(move || async move { get_record(rid).await });
        if let Some(record) = &*record.read() {
            let record = record.as_ref().unwrap();
            let Record {
                rid: _,
                pid,
                code,
                status,
            } = record;
            rsx! {
                p { "Problem {pid}" }
                p { "Status {status}" }
                code { "{code}" }
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
pub fn Record(rid: u64) -> Element {
    rsx! {
        record_page { rid }
    }
}
