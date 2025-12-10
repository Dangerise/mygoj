mod comp;

use shared::judge::JudgeSignal;
use std::sync::LazyLock;
use std::time::Duration;
use uuid::Uuid;

const SERVER_ORIGN: &'static str = "http://192.168.1.107:5800";
static UUID: LazyLock<Uuid> = LazyLock::new(|| Uuid::new_v4());

use reqwest::Client;
async fn send_signal() {
    let client = Client::new();
    let url = format!("{}/api/judge-signal", SERVER_ORIGN);
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
        client.post(url.clone()).json(&signal).send().await.unwrap();
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}

pub async fn main() -> eyre::Result<()> {
    tokio::spawn(send_signal());
    loop {
        tokio::time::sleep(Duration::from_millis(1000)).await;
    }
}
