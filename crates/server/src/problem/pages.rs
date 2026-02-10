use super::*;
use crate::db::DB;
use dashmap::DashMap;
use futures_util::StreamExt;
use sqlx::Row;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};

struct TimeCacheInner<T> {
    last_update: Instant,
    content: T,
}

struct TimeCache<T> {
    lock: RwLock<Option<TimeCacheInner<T>>>,
}

impl<T: Clone + Send + Sync> TimeCache<T> {
    pub const fn new() -> Self {
        TimeCache {
            lock: RwLock::const_new(None),
        }
    }
    pub fn read_or_update<Fut>(
        &self,
        update: impl Fn() -> Fut + Send + Sync,
    ) -> impl Future<Output = Result<T, ServerError>> + Send
    where
        Fut: Future<Output = Result<T, ServerError>> + Send,
    {
        async move {
            let now = Instant::now();
            let ret = loop {
                let read = self.lock.read().await;
                if read.is_none()
                    || read
                        .as_ref()
                        .is_some_and(|d| now.duration_since(d.last_update) > Duration::from_mins(1))
                {
                    drop(read);
                    let mut write = self.lock.write().await;
                    let content = update().await?;
                    *write = Some(TimeCacheInner {
                        last_update: now,
                        content,
                    });
                } else {
                    break read.as_ref().unwrap().content.clone();
                }
            };
            Ok(ret)
        }
    }
}

static PAGE_COUNT: TimeCache<u64> = TimeCache::new();

async fn db_get_page_count() -> Result<u64, ServerError> {
    let db = DB.get().unwrap();
    let row = sqlx::query("SELECT COUNT(*) FROM problems")
        .fetch_one(db)
        .await
        .map_err(ServerError::into_internal)?;
    let cnt: i64 = row.get(0);
    let cnt = (cnt as u64).div_ceil(PAGE_SIZE);
    Ok(cnt)
}

pub async fn get_page_count() -> Result<u64, ServerError> {
    PAGE_COUNT
        .read_or_update(async || db_get_page_count().await)
        .await
}

const PAGE_SIZE: u64 = 10;

async fn db_get_problems_page(index: u64) -> Result<Arc<Vec<Problem>>, ServerError> {
    let db = DB.get().unwrap();
    let offset = (index * PAGE_SIZE) as i64;
    let limit = PAGE_SIZE as i64;
    let mut stream = sqlx::query!(
        "SELECT json FROM problems LIMIT $1 OFFSET $2",
        limit,
        offset
    )
    .fetch(db);
    let mut ret = Vec::new();
    while let Some(item) = stream.next().await {
        let json = item
            .map_err(ServerError::into_internal)?
            .json
            .ok_or(ServerError::BadData)?;
        let p: Problem = serde_json::from_str(&json).map_err(|_| ServerError::BadData)?;
        ret.push(p);
    }
    Ok(Arc::new(ret))
}

#[dynamic]
static PROBLEMS_PAGES: DashMap<u64, Arc<TimeCache<Arc<Vec<Problem>>>>> = DashMap::new();
pub async fn get_problems_page(index: u64) -> Result<Arc<Vec<Problem>>, ServerError> {
    let lock = PROBLEMS_PAGES
        .entry(index)
        .or_insert_with(|| Arc::new(TimeCache::new()))
        .downgrade()
        .clone();
    lock.read_or_update(async || db_get_problems_page(index).await)
        .await
}
