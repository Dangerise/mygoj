use super::judge::JUDGE_QUEUE;
use super::problem::problem_read_lock;
use super::record::RECORDS;
use salvo::prelude::*;
use shared::record::{Record, RecordStatus, Rid};
use shared::submission::Submission;

#[handler]
pub async fn receive_submission(req: &mut Request, resp: &mut Response) -> eyre::Result<()> {
    let submission: Submission = req.parse_json().await?;
    tracing::info!("get submission {:?}", &submission);

    let rid;
    {
        let mut records = RECORDS.write().await;
        rid = Rid(records.len() as u64);
        records.push(Record {
            rid,
            pid: submission.pid.clone(),
            code: submission.code,
            status: RecordStatus::Waiting,
            timestamp: chrono::Utc::now().timestamp() as u64,
        });
    }

    tokio::spawn(async move {
        problem_read_lock(&submission.pid).await;
        let mut queue = JUDGE_QUEUE.lock().await;
        queue.push_back(rid);
    });

    resp.render(Json(&rid));
    Ok(())
}
