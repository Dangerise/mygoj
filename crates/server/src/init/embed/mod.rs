mod problems;

use super::*;
use crate::problem::Problem;
use rust_embed::RustEmbed;
use shared::problem::{Pid, ProblemFile, Testcase};
use shared::user::UserRegistration;
use uuid::Uuid;

pub async fn users() -> eyre::Result<()> {
    let author = UserRegistration {
        email: "dangerise@qq.com".into(),
        password: "1234".into(),
        nickname: "Dangerise".into(),
        username: "Dangerise".into(),
    };

    let uid = crate::user::user_register(author).await?;
    crate::user::set_su(uid).await?;

    Ok(())
}

pub async fn problems(path: impl AsRef<Path>) -> eyre::Result<()> {
    let path = path.as_ref();
    problems::embed_problem::<problems::ApB>(path).await?;
    #[cfg(debug_assertions)]
    {
        problems::embed_problem::<problems::ComplexFs>(path).await?;
        problems::embed_problem::<problems::SimpleFs>(path).await?;
    }
    Ok(())
}
