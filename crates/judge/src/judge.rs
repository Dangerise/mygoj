use super::*;

pub async fn judge(rid: Rid) -> eyre::Result<()> {
    let Record {
        rid,
        pid,
        code,
        status: _,
    } = accquire(format!("record?rid={}", rid.0)).await?;
    
    

    Ok(())
}
