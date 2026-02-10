mod problems;

use super::*;
use crate::problem::Problem;
use rust_embed::RustEmbed;
use shared::problem::{Pid, ProblemFile, Testcase};
use shared::user::UserRegistration;
use uuid::Uuid;

pub async fn users() -> eyre::Result<()> {
    let author1 = UserRegistration {
        email: "dangerise@qq.com".into(),
        password: "1234".into(),
        nickname: "Dangerise".into(),
        username: "dangerise".into(),
    };

    let uid = crate::user::user_register(author1).await?;
    crate::user::set_su(uid).await?;

    let author2 = UserRegistration {
        email: "2816055869".into(),
        password: "1234".into(),
        nickname: "skhuo".into(),
        username: "skhuo".into(),
    };

    let uid = crate::user::user_register(author2).await?;
    crate::user::set_su(uid).await?;

    let visitor = UserRegistration {
        email: "xxx@mygoj.ac".into(),
        password: "1234".into(),
        nickname: "visitor".into(),
        username: "visitor".into(),
    };

    crate::user::user_register(visitor).await?;

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
