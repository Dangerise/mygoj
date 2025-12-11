mod comp;
mod judge;

use serde::de::DeserializeOwned;
use shared::judge::*;
use shared::record::*;
use std::sync::LazyLock;
use std::time::Duration;
use uuid::Uuid;

const SERVER_ORIGN: &'static str = "http://192.168.1.107:5800";
static UUID: LazyLock<Uuid> = LazyLock::new(|| Uuid::new_v4());

async fn accquire<T>(path: String) -> eyre::Result<T>
where
    T: DeserializeOwned,
{
    let url = format!("{}/api/{}", SERVER_ORIGN, path);
    let res = reqwest::get(url).await?.json().await?;
    Ok(res)
}

async fn execute(command: Command) -> eyre::Result<()> {
    match command {
        Command::Judge(rid) => judge::judge(rid).await?,
        Command::Null => {}
    }
    Ok(())
}

use reqwest::Client;
async fn connect() {
    let client = Client::new();
    let url = format!("{}/api/judge/connect", SERVER_ORIGN);
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
        let signal = JudgeSignal {
            cpu_name: cpu_name.clone(),
            cpu_usage,
            system_name: system_name.clone(),
            hostname: hostname.clone(),
            tasks,
            uuid: *UUID,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
        };
        let command: Command = client
            .post(url.clone())
            .json(&signal)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        tokio::spawn(execute(command));
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}

pub async fn main() -> eyre::Result<()> {
    tokio::spawn(connect());
    loop {
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}
