use super::judge::judge_machines;
use super::problem::get_problem_front;
use super::record::get_record;
use super::submission::receive_submission;
use salvo::prelude::*;
use shared::front::FrontMessage;

#[handler]
pub async fn receive_front_message(req: &mut Request, resp: &mut Response) -> eyre::Result<()> {
    let message: FrontMessage = req.parse_json().await?;
    match message {
        FrontMessage::GetProblemFront(pid) => {
            let front = get_problem_front(&pid).await?;
            resp.render(Json(front));
        }
        FrontMessage::CheckJudgeMachines => {
            let res = judge_machines().await?;
            resp.render(Json(res));
        }
        FrontMessage::GetRecord(rid) => {
            let rec = get_record(rid).await?;
            resp.render(Json(rec));
        }
        FrontMessage::Submit(submission) => {
            let rid = receive_submission(submission).await?;
            resp.render(Json(rid));
        }
    }
    Ok(())
}
