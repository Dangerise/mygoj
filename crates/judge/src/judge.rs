use super::*;
use shared::problem::*;
use std::path::{Path, PathBuf};
use testbox::{PlatformTestBox, TestBox};
use tokio::{fs, process};

fn problem_file_path(pid: &Pid, file: &str) -> PathBuf {
    DIR.get().unwrap().join("problem").join(&pid.0).join(file)
}

#[instrument]
async fn sync_problem_file(pid: &Pid, file: &ProblemFile) -> eyre::Result<()> {
    let dir = DIR.get().unwrap().join("problem").join(&pid.0);
    if !dir.exists() {
        fs::create_dir_all(dir).await?;
    }

    let path = problem_file_path(pid, &file.path);
    let content: Vec<u8> =
        get_bin(JudgeMessage::GetProblemFile(pid.clone(), file.path.clone())).await?;
    tracing::info!("write to {}", path.display());
    fs::write(path, content).await?;
    Ok(())
}

#[instrument]
async fn prepare(data: &ProblemData) -> eyre::Result<()> {
    let ProblemData {
        pid,
        testcases,
        files,
        ..
    } = data;

    for case in testcases {
        for file in files {
            if file.path == case.input_file {
                sync_problem_file(pid, file).await?;
            }
            if file.path == case.output_file {
                sync_problem_file(pid, file).await?;
            }
        }
    }

    Ok(())
}

#[instrument]
async fn compile(dir: &Path, code: &str) -> eyre::Result<PathBuf> {
    tracing::info!("compile");

    let code_file = dir.join("prog.cpp");
    fs::write(&code_file, code).await?;
    let out_file = dir.join("prog");
    let output = process::Command::new("g++")
        .arg(&code_file)
        .arg("-o")
        .arg(&out_file)
        .arg("-O2")
        .arg("-std=c++14")
        .arg("-static")
        .output()
        .await?;
    if output.status.success() {
        Ok(out_file)
    } else {
        Err(CompileError {
            exit_code: output.status.code(),
            message: String::from_utf8(output.stderr)?,
        }
        .into())
    }
}

#[instrument]
async fn run_testcase(
    pid: &Pid,
    prog: &Path,
    time_limit: u32,
    memory_limit: u32,
    case: &Testcase,
) -> eyre::Result<SingleJudgeResult> {
    tracing::info!("running testcase");

    let testbox_dir = tempfile::TempDir::new()?;
    let testbox = PlatformTestBox::new(&testbox::Config {
        root: testbox_dir.path().into(),
        memory_limit: (memory_limit as u64) << 20,
        time_limit: Duration::from_millis(time_limit as u64),
    })
    .await?;

    let input = fs::read(problem_file_path(pid, &case.input_file)).await?;
    let run_result = testbox.run_single(prog, None, &input).await?;

    tracing::info!("run status {:?}", run_result.status);

    let mut ret = SingleJudgeResult {
        memory_used: (run_result.memory_used >> 20) as u32,
        time_used: run_result.time_used.as_millis() as u32,
        verdict: Verdict::Ac,
    };
    match run_result.status {
        testbox::Status::Okay => {}
        testbox::Status::TimeLimitExceed => {
            ret.verdict = Verdict::Tle;
            return Ok(ret);
        }
        testbox::Status::MemoryLimitExceed => {
            ret.verdict = Verdict::Mle;
            return Ok(ret);
        }
        testbox::Status::RuntimeError => {
            ret.verdict = Verdict::Re;
            return Ok(ret);
        }
    }

    let stdout = match String::from_utf8(run_result.stdout) {
        Ok(c) => c,
        Err(_) => {
            ret.verdict = Verdict::Wa;
            return Ok(ret);
        }
    };

    let answer = fs::read_to_string(problem_file_path(pid, &case.output_file)).await?;
    ret.verdict = if comp::comp(&answer, &stdout) {
        Verdict::Ac
    } else {
        Verdict::Wa
    };
    Ok(ret)
}

#[instrument]
pub async fn run_all_cases(
    rid: Rid,
    prog: &Path,
    problem_data: &ProblemData,
) -> eyre::Result<AllJudgeResult> {
    let mut memory = 0;
    let mut max_time = 0;
    let mut sum_time = 0;
    let mut verdict = Verdict::Ac;
    let mut cases_results = Vec::new();

    let mut cases = Vec::new();

    for (idx, case) in problem_data.testcases.iter().enumerate() {
        let case = case.clone();
        let prog = prog.to_path_buf();
        let time_limit = problem_data.time_limit;
        let memory_limit = problem_data.memory_limit;
        let pid = problem_data.pid.clone();
        let handle = tokio::spawn(async move {
            let res = run_testcase(&pid, &prog, time_limit, memory_limit, &case).await?;
            let _: () =
                send_message(JudgeMessage::SendSingleJudgeResult(rid, idx, res.clone())).await?;
            Ok::<_, eyre::Report>(res)
        });
        cases.push(handle);
    }

    for handle in cases {
        let res = handle.await.unwrap()?;
        memory = u32::max(memory, res.memory_used);
        max_time = u32::max(max_time, res.time_used);
        sum_time += res.time_used;
        if res.verdict.priority() > verdict.priority() {
            verdict = res.verdict;
        }
        cases_results.push(res);
    }

    Ok(AllJudgeResult {
        cases: cases_results,
        verdict,
        memory_used: memory,
        max_time,
        sum_time,
    })
}

#[instrument]
pub async fn judge(rid: Rid) -> eyre::Result<()> {
    let _enter = tracing::span!(
        tracing::Level::INFO,
        "begin to judge",
        rid = rid.to_string()
    );

    let record = send_message(JudgeMessage::GetRecord(rid)).await?;

    tracing::info!("receive record {:#?} ", record);

    let Record {
        rid: rid2,
        pid,
        code,
        ..
    } = record;

    assert_eq!(rid, rid2);

    let problem_data: ProblemData = send_message(JudgeMessage::GetProblemData(pid.clone())).await?;

    tracing::info!("receive problem data {:#?}", &problem_data);

    assert!(problem_data.check_unique());

    prepare(&problem_data).await?;

    let compile_dir = tempfile::TempDir::new()?;
    let prog = match compile(compile_dir.path(), &code).await {
        Ok(path) => path,
        Err(err) => {
            if let Some(ce) = err.downcast_ref::<CompileError>() {
                tracing::info!("compile error {:#?}", ce);
                let _: () = send_message(JudgeMessage::SendCompileResult(
                    rid,
                    CompileResult::Error(ce.clone()),
                ))
                .await?;
                return Ok(());
            }
            return Err(err);
        }
    };

    tracing::info!("compiled");
    let _: () = send_message(JudgeMessage::SendCompileResult(
        rid,
        CompileResult::Compiled,
    ))
    .await?;

    let res = run_all_cases(rid, &prog, &problem_data).await?;

    let _: () = send_message(JudgeMessage::SendAllJudgeResults(rid, res)).await?;

    Ok(())
}
