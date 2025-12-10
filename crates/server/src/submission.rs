use super::record::RECORDS;
use salvo::prelude::*;
use shared::record::{JudgeStatus, Record};
use shared::submission::Submission;

#[handler]
pub async fn receive_submission(req: &mut Request, resp: &mut Response) -> eyre::Result<()> {
    let submission: Submission = req.parse_json().await?;
    tracing::info!("get submission {:?}", &submission);

    let mut records = RECORDS.write().await;
    let rid = records.len() as u64;
    records.push(Record {
        rid,
        pid: submission.pid,
        code: submission.code,
        status: JudgeStatus::Judging,
    });

    resp.render(Json(&rid));
    Ok(())
}
