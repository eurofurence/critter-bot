use chrono::NaiveDate;
use color_eyre::eyre;
use futures_util::StreamExt;
use moka::future::Cache;
use sqlx::{PgPool, query, types::Json};
use std::{sync::Arc, time::Duration};
use teloxide::types::ChatId;
use tokio::sync::Semaphore;

use crate::events::Shift;

#[derive(serde::Deserialize)]
pub struct UserId(u64);

// I'm aware that the implementations I made here are wonderfully inefficient, but I really don't care for now, this will be reimplemented eventually (right?!)
#[derive(Clone)]
pub struct Database {
    pool: PgPool,
    c_cache: Cache<ChatId, Option<i64>>,
    u_cache: Cache<i64, Option<ChatId>>,
    lookup_limiter: Arc<Semaphore>,
}

impl Database {
    pub fn new(pool: PgPool, pq_limit: usize) -> Self {
        Self {
            pool,
            c_cache: Cache::builder()
                .time_to_idle(Duration::from_secs(300))
                .build(),
            u_cache: Cache::builder()
                .time_to_idle(Duration::from_secs(3600 * 6))
                .build(),
            lookup_limiter: Arc::new(Semaphore::new(pq_limit)),
        }
    }

    pub async fn check_if_present(&self, cid: ChatId) -> eyre::Result<Option<i64>> {
        if let Some(res) = self.c_cache.get(&cid).await {
            return Ok(res);
        }
        let res = query!("select id from critters where tgid = $1", cid.0)
            .fetch_optional(&self.pool)
            .await?
            .map(|rec| rec.id);
        self.c_cache.insert(cid, res).await;
        Ok(res)
    }

    pub async fn register(&self, uid: i64, cid: ChatId) -> eyre::Result<()> {
        query!(
            "insert into critters (id, tgid) values ($1, $2)",
            uid,
            cid.0
        )
        .execute(&self.pool)
        .await?;

        self.c_cache.insert(cid, Some(uid)).await;
        self.u_cache.insert(uid, Some(cid)).await;

        Ok(())
    }

    pub async fn get_chat_id(&self, uid: i64) -> eyre::Result<Option<ChatId>> {
        if let Some(res) = self.u_cache.get(&uid).await {
            return Ok(res);
        }
        let _ = self.lookup_limiter.acquire().await;

        let res = query!("select tgid from critters where id = $1", uid as i64)
            .fetch_optional(&self.pool)
            .await?
            .map(|rec| ChatId(rec.tgid));

        self.u_cache.insert(uid, res).await;

        Ok(res)
    }

    // pub async fn sync_shift(&self, shift: Shift) -> eyre::Result<Option<SyncDiff>> {
    //     let mut tx = self.pool.begin().await?;

    //     let Some(meta): Option<Shift> = query!(
    //         "select meta as \"meta: Json<Shift>\" from shifts where id = $1",
    //         shift.id
    //     )
    //     .fetch_optional(&mut *tx)
    //     .await?
    //     .map(|rec| rec.meta.0) else {
    //         todo!("create shift entry");

    //         return Ok(Some(SyncDiff::Created));
    //     };
    //     if shift == meta {
    //         return Ok(None);
    //     }

    //     query!(
    //         "update shifts set meta = $1 where id = $2",
    //         serde_json::to_value(&shift)?,
    //         shift.id,
    //     )
    //     .execute(&mut *tx)
    //     .await?;

    //     if shift.start != meta.start || shift.end != meta.end {
    //         query!(
    //             "update shifts set start = $1, stop = $2 where id = $3",
    //             shift.start.naive_utc(),
    //             shift.end.naive_utc(),
    //             shift.id,
    //         )
    //         .execute(&mut *tx)
    //         .await?;
    //     }

    //     todo!()
    // }

    pub async fn posts(&self, date: NaiveDate) -> eyre::Result<Vec<Shift>> {
        let mut stream = query!(
            "select meta as \"meta: Json<Shift>\" from shifts where date(start) = $1",
            date
        )
        .fetch(&self.pool);
        let mut shifts = Vec::new();
        while let Some(shift) = stream.next().await {
            shifts.push(shift?.meta.0);
        }
        Ok(shifts)
    }

    pub async fn sync_dates(&self, cur_dates: &[NaiveDate]) -> eyre::Result<()> {
        let dates = query!("select \"date\" from dates")
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|d| d.date)
            .collect::<Vec<_>>();
        let missing = cur_dates.iter().filter(|d| dates.binary_search(d).is_err());
        let invalid = dates.iter().filter(|d| cur_dates.binary_search(d).is_err());

        for m in missing {
            query!("insert into dates (\"date\") values ($1)", m)
                .execute(&self.pool)
                .await?;
        }
        for i in invalid {
            query!("delete from dates where \"date\" = $1", i)
                .execute(&self.pool)
                .await?;
        }

        Ok(())
    }
}
