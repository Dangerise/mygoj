use std::fs;
use std::process::Command;
use std::time::Duration;
use testbox::*;

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
    let out = run(include_str!("normal.cpp"), 20, 1000, "1 2").await;
    println!("{:?}", out);
    println!("stdout {}", String::from_utf8_lossy(&out.stdout));
    assert_eq!(out.status, Status::Okay);
    assert_eq!(out.stdout.as_slice(), "3\n".as_bytes());
}

#[tokio::test]
async fn vector_memory_enough() {
    let out = run(include_str!("vector_memory.cpp"), 128, 1000, "100").await;
    println!("stdout {}", String::from_utf8_lossy(&out.stdout));
    println!("{:?}", out);
    assert_eq!(out.status, Status::Okay);
    assert_eq!(out.exit_code, Some(0));
}

#[tokio::test]
async fn vector_memory_out() {
    let out = run(include_str!("vector_memory.cpp"), 128, 1000, "200").await;
    println!("stdout {}", String::from_utf8_lossy(&out.stdout));
    println!("{:?}", out);
    assert_eq!(out.status, Status::MemoryLimitExceed);
}

#[tokio::test]
async fn array_memory_out() {
    let out = run(include_str!("array_memory.cpp"), 100, 1000, "").await;
    println!("stdout {}", String::from_utf8_lossy(&out.stdout));
    println!("{:?}", out);
    assert_eq!(out.status, Status::MemoryLimitExceed);
    assert_eq!(out.exit_code, Some(0));
}

#[tokio::test]
async fn timeout() {
    let out = run(include_str!("timeout.cpp"), 100, 4, "").await;
    println!("stdout {}", String::from_utf8_lossy(&out.stdout));
    println!("{:?}", out);
    assert_eq!(out.status, Status::TimeLimitExceed);
}
