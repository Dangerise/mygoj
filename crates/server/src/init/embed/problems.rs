use super::*;

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
        base.files.push(ProblemFile {
            path: file.into(),
            uuid: Uuid::new_v4(),
        });
    }
    base
}

pub async fn write_fs<P: EmbedProblem>(path: &Path) -> eyre::Result<()> {
    for filename in P::iter() {
        let path = path
            .join("problems")
            .join(ApB::base().pid.0)
            .join(&*filename);
        let content = &*P::get(&*filename).unwrap().data;
        fs::write(&path, content).await?;
    }
    Ok(())
}
