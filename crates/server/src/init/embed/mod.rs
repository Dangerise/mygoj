mod problems;

use super::*;
use crate::problem::Problem;
use problems::ApB;
use rust_embed::RustEmbed;
use shared::problem::{Pid, ProblemFile, Testcase};
use shared::user::UserRegistration;
use uuid::Uuid;

pub async fn with_db() -> eyre::Result<()> {
    let author = UserRegistration {
        email: "dangerise@qq.com".into(),
        password: "1234".into(),
        nickname: "Dangerise".into(),
        username: "Dangerise".into(),
    };

    crate::user::user_register(author).await?;

    let db = DB.get().unwrap();
    problems::generate::<ApB>().insert_db(db).await?;

    Ok(())
}

pub async fn with_fs(path: &Path) -> eyre::Result<()> {
    problems::write_fs::<problems::ApB>(path).await?;
    Ok(())
}
