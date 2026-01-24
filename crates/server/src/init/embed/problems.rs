use super::*;

#[derive(RustEmbed)]
#[folder = "src/init/embed/simple_fs"]
pub struct SimpleFs;

impl EmbedProblem for SimpleFs {
    fn base() -> Problem {
        Problem {
            pid: Pid::new("3"),
            owner: None,
            title: "simple_fs".into(),
            statement: "the statement of simplefs".to_string().into(),
            memory_limit: 0,
            time_limit: 0,
            testcases: vec![].into(),
            files: vec![].into(),
        }
    }
}

#[derive(RustEmbed)]
#[folder = "src/init/embed/complex_fs"]
pub struct ComplexFs;

impl EmbedProblem for ComplexFs {
    fn base() -> Problem {
        Problem {
            pid: Pid::new("2"),
            owner: None,
            title: "complex_fs".into(),
            statement: "none statment".to_string().into(),
            memory_limit: 2,
            time_limit: 100,
            testcases: vec![].into(),
            files: vec![].into(),
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
            statement: include_str!("a+b/statement.md").to_string().into(),
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
            ]
            .into(),
            files: vec![].into(),
        }
    }
}

pub trait EmbedProblem: RustEmbed {
    fn base() -> Problem;
}

pub async fn embed_problem<P: EmbedProblem>(path: &Path) -> eyre::Result<()> {
    let p = generate::<P>();
    let db = DB.get().unwrap();
    p.insert_db(db).await?;
    write_fs::<P>(&path.join(p.pid.0.as_str()), &p).await?;
    Ok(())
}

pub fn generate<P: EmbedProblem>() -> Problem {
    let mut base = P::base();
    let mut files = Vec::new();
    for file in P::iter() {
        let file = &*file;
        let inner = P::get(file).unwrap();
        let size = inner.data.len() as u64;
        files.push(ProblemFile {
            path: file.into(),
            uuid: Uuid::new_v4(),
            last_modified: inner.metadata.last_modified().unwrap() as i64,
            size,
            is_public: false,
        });
    }
    base.files = files.into();
    base
}

pub async fn write_fs<P: EmbedProblem>(path: &Path, g: &Problem) -> eyre::Result<()> {
    let dir = path;
    if !fs::try_exists(&dir).await? {
        fs::create_dir_all(&dir).await?;
    }
    let files = &g.files;
    for filename in P::iter() {
        let uuid = files
            .iter()
            .find_map(|d| (d.path == filename).then_some(d.uuid))
            .unwrap();
        let path = dir.join(uuid.to_string());
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        let content = &*P::get(&filename).unwrap().data;
        fs::write(&path, content).await?;
        tracing::trace!("write problem file {} {}", filename, path.display());
    }
    Ok(())
}
