use super::*;
use std::ffi::OsString;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use tokio::fs;
use wait4::Wait4;

#[derive(Debug, Clone)]
pub struct LinuxTestBox {
    config: Config,
}

impl TestBox for LinuxTestBox {
    async fn new(config: &Config) -> Result<Self> {
        Ok(LinuxTestBox {
            config: config.clone(),
        })
    }
    async fn run_single<'a>(
        &self,
        path: impl AsRef<Path>,
        args: impl IntoIterator<Item = &'a OsStr>,
        stdin: impl AsRef<[u8]>,
    ) -> Result<RunResult> {
        let path = path.as_ref();
        if self.config.root.exists() {
            fs::remove_dir_all(&self.config.root).await?;
        }
        fs::create_dir_all(&self.config.root).await?;
        fs::copy(path, &self.config.root.join("prog")).await?;

        let mut private = OsString::from("--private=");
        private.push(&self.config.root);

        let start = std::time::Instant::now();

        let mut command = Command::new("firejail");

        command
            .arg(&private)
            .arg(format!("--rlimit-as={}", self.config.memory_limit * 2))
            .arg(format!(
                "--rlimit-cpu={}",
                self.config.time_limit.as_secs_f64().ceil() as u32
            ))
            .arg("--rlimit-nproc=4")
            .arg("./prog");

        for item in args {
            command.arg(item);
        }

        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        let pid = child.id();

        let stdin_data = stdin.as_ref().to_owned();
        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = child.stdout.take().unwrap();

        let stdin = tokio::task::spawn_blocking(move || stdin.write_all(&stdin_data));
        let stdout = tokio::task::spawn_blocking(move || {
            let mut data = Vec::new();
            stdout.read_to_end(&mut data)?;
            Ok::<_, std::io::Error>(data)
        });

        let proc = tokio::task::spawn_blocking(move || child.wait4());

        let mut status = Status::Okay;

        let pid = nix::unistd::Pid::from_raw(pid as i32);
        let kill = async || {
            let sigkill = nix::sys::signal::SIGKILL;
            tokio::task::spawn_blocking(move || nix::sys::signal::kill(pid, sigkill))
                .await
                .map_err(map_err)??;
            Ok::<_, Error>(())
        };

        let mut system = sysinfo::System::new();
        while !proc.is_finished() {
            if start.elapsed() > self.config.time_limit {
                tracing::info!("manual kill for time");
                kill().await?;
                status = Status::TimeLimitExceed;
                break;
            }
            let memory = system.refresh_processes_specifics(
                sysinfo::ProcessesToUpdate::Some(&[sysinfo::Pid::from_u32(pid.as_raw() as u32)]),
                false,
                sysinfo::ProcessRefreshKind::nothing().with_memory(),
            );
            if memory as u64 > self.config.memory_limit {
                tracing::info!("manual kill for memory");
                kill().await?;
                status = Status::MemoryLimitExceed;
                break;
            }
            // tokio::time::sleep(Duration::from_millis(100)).await;
        }

        stdin.await.map_err(map_err)??;
        let stdout = stdout.await.map_err(map_err)??;

        let res = proc.await.map_err(map_err)??;

        if res.rusage.maxrss > self.config.memory_limit {
            status = Status::MemoryLimitExceed;
        } else if res.status.code() != Some(0) && status == Status::Okay {
            status = Status::RuntimeError;
        }

        let wall_time = start.elapsed();
        tracing::info!("wall time {}", wall_time.as_millis());

        Ok(RunResult {
            time_used: res.rusage.stime,
            memory_used: res.rusage.maxrss,
            exit_code: res.status.code(),
            status,
            stdout,
        })
    }
}
