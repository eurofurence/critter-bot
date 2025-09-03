use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use color_eyre::eyre::{self, WrapErr};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    sync::Arc,
    time::Duration,
};
use teloxide::{prelude::Requester, types::ChatId};
use tokio::{sync::mpsc::UnboundedReceiver, time::sleep};
use tracing::{debug, error, trace};

use crate::State;

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Shift {
    pub id: i64,
    pub title: Arc<str>,
    pub r#type: Arc<str>,
    pub location: Arc<str>,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub tz: Tz,
    pub critters: Vec<(Arc<str>, Arc<str>, i64, bool)>,
    pub managers: Vec<(Arc<str>, i64)>,
    pub req: usize,
    pub ppe: bool,
}

pub enum Event {
    UserUpcoming {
        uid: i64,
        shift: Shift,
    },
    UserTimeChanged {
        uid: i64,
        shift: Shift,
        old_start: DateTime<Tz>,
        old_end: DateTime<Tz>,
    },
    UserCanceled {
        uid: i64,
        shift: Shift,
    },
    /// TODO: wait for rusty
    ManagerUpcoming {
        uid: i64,
        shift: Shift,
    },
    UserDaily {
        uid: i64,
        next: Vec<Shift>,
    },
}

#[derive(Debug)]
pub enum ShiftDiff {
    Created,
    Updated,
    TimeUpdated {
        old_start: DateTime<Utc>,
        old_end: DateTime<Utc>,
    },
    Deleted,
}

#[tracing::instrument(name = "event_poll", skip(state))]
pub async fn start_event_processor(state: State) -> eyre::Result<()> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(distribute(state.clone(), rx));

    loop {
        trace!("polling data...");

        trace!("syncing dates...");
        let dates = state.api.dates().await.wrap_err("api dates")?;
        state.db.sync_dates(&dates).await.wrap_err("db date sync")?;

        let date = Utc::now().with_timezone(&state.tz).date_naive();
        trace!(date = date.to_string(), "syncing posts of the day...");
        let old = state.db.posts(date).await.wrap_err("db posts pull")?;
        let new = state
            .api
            .shifts(date, state.tz)
            .await
            .wrap_err("api posts")?;

        let now = Utc::now();
        for (shift, change) in scan_iter(&old, &new) {
            debug!("{change:?} - {}", shift.id);
            match change {
                Option::None => (),
                Some(ShiftDiff::Created) => {
                    state
                        .db
                        .insert_shift(shift)
                        .await
                        .wrap_err("create shift")?;
                }
                Some(ShiftDiff::Updated) => {
                    state
                        .db
                        .update_shift(shift)
                        .await
                        .wrap_err("update shift")?;
                }
                Some(ShiftDiff::TimeUpdated { old_start, old_end }) => {
                    state
                        .db
                        .update_shift(shift)
                        .await
                        .wrap_err("update shift + time")?;
                    for c in &shift.critters {
                        tx.send(Event::UserTimeChanged {
                            uid: c.2,
                            shift: shift.clone(),
                            old_start: old_start.with_timezone(&state.tz),
                            old_end: old_end.with_timezone(&state.tz),
                        });
                    }
                }
                Some(ShiftDiff::Deleted) => {
                    state
                        .db
                        .delete_shift(shift.id)
                        .await
                        .wrap_err("delete shift")?;
                    for c in &shift.critters {
                        tx.send(Event::UserCanceled {
                            uid: c.2,
                            shift: shift.clone(),
                        });
                    }
                    continue;
                }
            }
            if now.signed_duration_since(shift.start).num_minutes().abs() < 15
                && !state.db.has_been_notified(shift.id).await?
            {
                for c in &shift.critters {
                    tx.send(Event::UserUpcoming {
                        uid: c.2,
                        shift: shift.clone(),
                    });
                }
                state.db.notify(shift.id, true).await?;
            }
        }

        if let Some(false) = state
            .db
            .has_day_been_notified(now.with_timezone(&state.tz).date_naive())
            .await?
        {
            // TODO: inform user of daily stuff
        }

        sleep(Duration::from_secs(state.poll_interval as u64)).await;
    }
}

/// We do some early filtering here to not spawn unnececary task (not like tokio would care), but also to not overwhelm the db, not that this software will ever run on scale haha
pub async fn distribute(state: State, mut stream: UnboundedReceiver<Event>) {
    while let Some(event) = stream.recv().await {
        let Some(cid) = crate::retry!(state.db.get_chat_id(event.target())) else {
            continue;
        };
        let state = state.clone();
        tokio::spawn(async move {
            let Err(err) = handle_event(state, event, cid).await else {
                return;
            };
            error!("{err}");
        });
    }
}

#[tracing::instrument(name = "bot_event_send", skip(state, event))]
pub async fn handle_event(state: State, event: Event, cid: ChatId) -> eyre::Result<()> {
    let Err(err) = state.bot.send_message(cid, format!("{}", event)).await else {
        trace!("send reminder");
        return Ok(());
    };
    if matches!(
        err,
        teloxide::RequestError::Api(teloxide::ApiError::BotBlocked)
    ) {
        trace!("bot is blocked");
        return Ok(());
    };

    Err(err)?
}

impl Event {
    fn target(&self) -> i64 {
        match self {
            Event::UserUpcoming { uid, .. } => *uid,
            Event::ManagerUpcoming { uid, .. } => *uid,
            Event::UserDaily { uid, .. } => *uid,
            Event::UserTimeChanged { uid, .. } => *uid,
            Event::UserCanceled { uid, .. } => *uid,
        }
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::UserUpcoming { shift, uid } => {
                writeln!(
                    f,
                    "**Upcoming shift:** {} ({}) as {}",
                    shift.title,
                    shift.r#type,
                    shift.critters.iter().find(|c| c.2 == *uid).unwrap().1
                )?;
                writeln!(f, "Location: {}", shift.location)?;
                writeln!(
                    f,
                    "Starts: {} (in {})",
                    shift.start.with_timezone(&shift.tz),
                    shift.start.signed_duration_since(&Utc::now())
                )?;
                writeln!(
                    f,
                    "Ends: {} ({} total)",
                    shift.end.with_timezone(&shift.tz),
                    shift.end.signed_duration_since(&shift.start)
                )?;

                if shift.ppe {
                    writeln!(f, "**PPE is required**")?;
                }

                Ok(())
            }
            Event::ManagerUpcoming { shift, .. } => {
                writeln!(f, "**Upcoming shift:** {} ({})", shift.title, shift.r#type)?;
                writeln!(f, "Location: {}", shift.location)?;
                writeln!(
                    f,
                    "Starts: {} (in {})",
                    shift.start.with_timezone(&shift.start.timezone()),
                    shift.start.signed_duration_since(&Utc::now())
                )?;
                writeln!(
                    f,
                    "Ends: {} ({} total)",
                    shift.end.with_timezone(&shift.start.timezone()),
                    shift.end.signed_duration_since(&shift.start)
                )?;

                if shift.ppe {
                    writeln!(f, "**PPE is required**")?;
                }

                writeln!(
                    f,
                    "Assigned critters: ({}/{})",
                    shift.critters.len(),
                    shift.req
                )?;
                for (name, r#type, _, staff) in shift.critters.iter() {
                    writeln!(
                        f,
                        "- {}{} as {}",
                        name,
                        if *staff { " (Staff)" } else { "" },
                        r#type
                    )?;
                }

                Ok(())
            }
            Event::UserDaily { next, uid } => {
                writeln!(f, "Your shifts today: ")?;

                for shift in next {
                    writeln!(
                        f,
                        "- **{} ({}) as {}** @ {}: {} (in {}) -> {} ({} total){}",
                        shift.title,
                        shift.r#type,
                        shift.critters.iter().find(|c| c.2 == *uid).unwrap().1,
                        shift.location,
                        shift.start.with_timezone(&shift.start.timezone()),
                        shift.start.signed_duration_since(&Utc::now()),
                        shift.end.with_timezone(&shift.start.timezone()),
                        shift.end.signed_duration_since(&shift.start),
                        if shift.ppe { " **[PPE]**" } else { "" }
                    )?;
                }

                Ok(())
            }
            Event::UserTimeChanged {
                uid,
                shift,
                old_start,
                old_end,
            } => {
                writeln!(
                    f,
                    "**Starttime of shift changed:** {} ({}) as {}",
                    shift.title,
                    shift.r#type,
                    shift.critters.iter().find(|c| c.2 == *uid).unwrap().1
                )?;
                writeln!(
                    f,
                    "Now Starts: **{} (in {})**, originally {}",
                    shift.start.with_timezone(&shift.start.timezone()),
                    shift.start.signed_duration_since(&Utc::now()),
                    old_start.with_timezone(&shift.start.timezone()),
                )?;
                writeln!(
                    f,
                    "Ends: **{} ({} total)**, originally {}",
                    shift.end.with_timezone(&shift.start.timezone()),
                    shift.end.signed_duration_since(&shift.start),
                    old_end.with_timezone(&shift.start.timezone()),
                )?;

                Ok(())
            }
            Event::UserCanceled { uid, shift } => {
                writeln!(
                    f,
                    "**Shift canceled:** {} ({}) as {}",
                    shift.title,
                    shift.r#type,
                    shift.critters.iter().find(|c| c.2 == *uid).unwrap().1
                )?;
                writeln!(
                    f,
                    "You no longer need to show up.\n\nIf you believe this was a mistake please contact the responsible shift manager."
                )?;
                Ok(())
            }
        }
    }
}

pub fn diff_shift(old: Option<&Shift>, new: Option<&Shift>) -> Option<ShiftDiff> {
    debug!("{} -> {}", old.is_some(), new.is_some());
    if old == new {
        return None;
    }
    let Some(old) = old else {
        return Some(ShiftDiff::Created);
    };
    let Some(new) = new else {
        return Some(ShiftDiff::Deleted);
    };
    if old.start != new.start || old.end != new.end {
        Some(ShiftDiff::TimeUpdated {
            old_start: old.start,
            old_end: old.end,
        })
    } else {
        Some(ShiftDiff::Updated)
    }
}

fn scan_iter<'a>(
    old: &'a [Shift],
    new: &'a [Shift],
) -> impl Iterator<Item = (&'a Shift, Option<ShiftDiff>)> {
    let mut keys = HashSet::new();
    let old = old
        .into_iter()
        .map(|s| {
            keys.insert(s.id);
            s
        })
        .map(|s| (s.id, s))
        .collect::<HashMap<_, _>>();
    let new = new
        .into_iter()
        .map(|s| {
            keys.insert(s.id);
            s
        })
        .map(|s| (s.id, s))
        .collect::<HashMap<_, _>>();

    keys.into_iter().map(move |id| {
        let old = old.get(&id);
        let new = new.get(&id);
        let act = new.or(old).unwrap();

        (*act, diff_shift(old.map(|s| *s), new.map(|s| *s)))
    })
}
