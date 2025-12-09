use super::*;
use std::ffi::OsString;
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
        fs::write(self.config.root.join("stdin"), stdin).await?;
        fs::copy(path, &self.config.root.join("prog")).await?;

        let mut private = OsString::from("--private=");
        private.push(&self.config.root);

        let mut run_command = OsString::from("./prog ");
        for item in args {
            run_command.push(item);
            run_command.push(" ");
        }
        run_command.push("< stdin > stdout");
        
        dbg!(&run_command);

        let start = std::time::Instant::now();

        let mut child = Command::new("firejail")
            .arg(&private)
            .arg(&format!("--rlimit-as={}", self.config.memory_limit))
            .arg(&format!(
                "--rlimit-cpu={}",
                self.config.time_limit.as_secs_f64().ceil() as u32
            ))
            .arg(&format!("--rlimit-nproc=4"))
            .arg("bash")
            .arg("-c")
            .arg(&run_command)
            .stdout(Stdio::null())
            .stdin(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        let pid = child.id();

        let proc = tokio::task::spawn_blocking(move || child.wait4());

        while !proc.is_finished() {
            if start.elapsed() > self.config.time_limit {
                let pid = nix::unistd::Pid::from_raw(pid as i32);
                let sigkill = nix::sys::signal::SIGKILL;
                tokio::task::spawn_blocking(move || nix::sys::signal::kill(pid, sigkill))
                    .await
                    .map_err(map_err)??;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        let res = proc.await.map_err(map_err)??;

        let stdout = fs::read(self.config.root.join("stdout")).await?;

        Ok(RunResult {
            time_used: res.rusage.stime,
            memory_used: res.rusage.maxrss,
            exit_code: res.status.code(),
            stdout,
        })
    }
}
