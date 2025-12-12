mod comp;
mod judge;

use serde::de::DeserializeOwned;
use shared::judge::*;
use shared::record::*;
use std::path::PathBuf;
use std::sync::{LazyLock, OnceLock};
use std::time::Duration;
use tokio::fs;
use tracing::instrument;
use uuid::Uuid;

static DIR: OnceLock<PathBuf> = OnceLock::new();

static UUID: LazyLock<Uuid> = LazyLock::new(Uuid::new_v4);

const SERVER_ORIGN: &str = "http://localhost:5800";

async fn send_message<T>(msg: JudgeMessage) -> eyre::Result<T>
where
    T: DeserializeOwned,
{
    let res = Client::new()
        .get(format!("{}/api/judge", SERVER_ORIGN))
        .json(&msg)
        .send()
        .await?
        .json()
        .await?;
    Ok(res)
}

async fn get_bin(msg: JudgeMessage) -> eyre::Result<Vec<u8>> {
    let res = Client::new()
        .get(format!("{}/api/judge", SERVER_ORIGN))
        .json(&msg)
        .send()
        .await?
        .bytes()
        .await?;
    let res = &*res;
    Ok(Vec::from(res))
}

async fn execute(command: JudgeCommand) -> eyre::Result<()> {
    match command {
        JudgeCommand::Judge(rid) => judge::judge(rid).await.unwrap(),
        JudgeCommand::Null => {}
    }
    Ok(())
}

use reqwest::Client;
async fn connect() {
    let mut system = sysinfo::System::new_all();
    let cpus = system.cpus();
    dbg!(&cpus);
    let cpu_name = cpus.iter().fold(String::new(), |mut a, b| {
        a.push_str(&format!("[{}] {} MHz ", b.name(), b.frequency()));
        a
    });
    let system_name = sysinfo::System::name();
    let hostname = sysinfo::System::host_name();
    loop {
        system.refresh_all();
        let cpu_usage = system.global_cpu_usage() as u32;
        let tasks = Vec::new();
        let signal = JudgeMachineSignal {
            cpu_name: cpu_name.clone(),
            cpu_usage,
            system_name: system_name.clone(),
            hostname: hostname.clone(),
            tasks,
            uuid: *UUID,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
        };
        let command: JudgeCommand = send_message(JudgeMessage::Signal(signal)).await.unwrap();
        tokio::spawn(execute(command));
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}

pub async fn main() -> eyre::Result<()> {
    tracing_subscriber::fmt().init();

    DIR.set(dirs::home_dir().unwrap().join("mygoj_judge")).unwrap();

    let dir = DIR.get().unwrap();
    if !dir.exists() {
        fs::create_dir_all(dir).await.unwrap();
    }

    tokio::spawn(connect());
    loop {
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}
