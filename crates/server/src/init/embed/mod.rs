mod problems;

use super::*;
use crate::problem::Problem;
use problems::ApB;
use rust_embed::RustEmbed;
use shared::problem::{Pid, ProblemFile, Testcase};
use uuid::Uuid;

pub async fn with_db(pool: &SqlitePool) -> eyre::Result<()> {
    // let author =  {
    //     email: "dangerise@qq.com".into(),
    //     password: "1234".into(),
    //     created_time: 0,
    //     nickname: "Dangerise".into(),
    //     username: "Dangerise".into(),
    //     uid: Uid(1),
    // };

    problems::generate::<ApB>().insert_db(pool).await?;

    Ok(())
}

pub async fn with_fs(path: &Path) -> eyre::Result<()> {
    problems::write_fs::<problems::ApB>(path).await?;
    Ok(())
}
