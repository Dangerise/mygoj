use salvo::prelude::*;
use shared::judge::*;
use shared::record::Rid;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::LazyLock;
use std::time::Duration;
use tokio::sync::Mutex;
use uuid::Uuid;

pub static JUDGE_QUEUE: LazyLock<Mutex<VecDeque<Rid>>> =
    LazyLock::new(|| Mutex::new(VecDeque::new()));

static SIGNALS: LazyLock<Mutex<HashMap<Uuid, JudgeSignal>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub async fn check_alive() {
    let mut to_remove = Vec::new();
    loop {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        let mut signals = SIGNALS.lock().await;
        to_remove.clear();
        for sig in signals.values() {
            if now - sig.timestamp > 4000 {
                to_remove.push(sig.uuid);
            }
        }
        for uuid in &to_remove {
            signals.remove(uuid);
            tracing::info!("judge machine offline {}", uuid);
        }
        drop(signals);
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

#[handler]
pub async fn connect(req: &mut Request, resp: &mut Response) -> eyre::Result<()> {
    let signal: JudgeSignal = req.parse_json().await?;
    let uuid = signal.uuid;
    let mut signals = SIGNALS.lock().await;
    tracing::info!("received signal {:?}", &signal);
    if signals.contains_key(&uuid) {
        *signals.get_mut(&uuid).unwrap() = signal;
    } else {
        tracing::info!("new judge machine online {}", uuid);
        signals.insert(uuid, signal);
    }
    resp.render(Json(Command::Null));
    Ok(())
}

#[handler]
pub async fn judge_machines(_req: &mut Request, resp: &mut Response) -> eyre::Result<()> {
    resp.render(Json(
        SIGNALS
            .lock()
            .await
            .iter()
            .map(|x| x.1.clone())
            .collect::<Vec<_>>(),
    ));
    Ok(())
}
