use std::fs;
use std::process::Command;
use std::time::Duration;
use testbox::{Config, PlatformTestBox, RunResult, TestBox};

async fn run(code: &str, memory: u64, time: u64, stdin: &str) -> RunResult {
    if fs::exists("tmp").unwrap() {
        fs::remove_dir_all("tmp").unwrap();
    }
    fs::create_dir_all("tmp").unwrap();
    fs::write("tmp/prog.cpp", code).unwrap();
    let status = Command::new("g++")
        .arg("tmp/prog.cpp")
        .arg("-o")
        .arg("tmp/prog")
        .status()
        .unwrap();
    assert!(status.success());

    let testbox = PlatformTestBox::new(&Config {
        root: "testbox".into(),
        memory_limit: memory << 20,
        time_limit: Duration::from_millis(time),
    })
    .await
    .unwrap();
    let out = testbox.run_single("tmp/prog", None, stdin).await.unwrap();
    out
}

#[tokio::test]
async fn normal() {
    let out = run(include_str!("normal.cpp"), 20, 100, "1 2").await;
    assert_eq!(out.exit_code, Some(13));
    assert_eq!(out.stdout.as_slice(), "3\n".as_bytes());
    println!("{:?}", out);
}

#[tokio::test]
async fn enough_memory() {
    let out = run(include_str!("memory.cpp"), 128, 100, "100").await;
    println!("{:?}", out);
    assert_eq!(out.exit_code, Some(0));
}
