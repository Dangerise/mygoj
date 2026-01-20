use super::*;

#[derive(RustEmbed)]
#[folder = "src/init/embed/complex_fs"]
pub struct ComplexFs;

impl EmbedProblem for ComplexFs {
    fn base() -> Problem {
        Problem {
            pid: Pid::new("2"),
            owner: None,
            title: "complex_fs".into(),
            statement: "none statment".into(),
            memory_limit: 2,
            time_limit: 100,
            testcases: vec![],
            files: vec![],
        }
    }
}

#[derive(RustEmbed)]
#[folder = "src/init/embed/a+b"]
pub struct ApB;

impl EmbedProblem for ApB {
    fn base() -> Problem {
        Problem {
            pid: Pid::new("1"),
            owner: None,
            title: "A+B".into(),
            statement: include_str!("a+b/statement.md").to_string(),
            memory_limit: 512,
            time_limit: 1000,
            testcases: vec![
                Testcase {
                    input_file: "1.in".into(),
                    output_file: "1.out".into(),
                },
                Testcase {
                    input_file: "2.in".into(),
                    output_file: "2.out".into(),
                },
            ],
            files: vec![],
        }
    }
}

pub trait EmbedProblem: RustEmbed {
    fn base() -> Problem;
}

pub fn generate<P: EmbedProblem>() -> Problem {
    let mut base = P::base();
    for file in P::iter() {
        let file = &*file;
        let inner = P::get(file).unwrap();
        let size = inner.data.len() as u64;
        base.files.push(ProblemFile {
            path: file.into(),
            uuid: Uuid::new_v4(),
            last_modified: inner.metadata.last_modified().unwrap() as i64,
            size,
            is_public: false,
        });
    }
    base
}

pub async fn write_fs<P: EmbedProblem>(path: &Path) -> eyre::Result<()> {
    let dir = path.join("problems").join(P::base().pid.0);
    if !fs::try_exists(&dir).await? {
        fs::create_dir_all(&dir).await?;
    }
    for filename in P::iter() {
        let path = dir.join(&*filename);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        let content = &*P::get(&filename).unwrap().data;
        fs::write(&path, content).await?;
        tracing::trace!("write problem file {}", path.display());
    }
    Ok(())
}
