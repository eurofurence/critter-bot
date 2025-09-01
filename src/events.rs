use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use color_eyre::eyre;
use std::{fmt::Display, sync::Arc};
use teloxide::{prelude::Requester, types::ChatId};
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::error;

use crate::State;

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
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
    /// eligibility/needs_cert
    pub ppe: bool,
}

pub enum Event {
    UserUpcoming {
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

pub async fn start_event_processor(state: State) -> eyre::Result<()> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(distribute(state.clone(), rx));

    loop {}

    Ok(())
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

pub async fn handle_event(state: State, event: Event, cid: ChatId) -> eyre::Result<()> {
    let Err(err) = state.bot.send_message(cid, format!("{}", event)).await else {
        return Ok(());
    };
    if matches!(
        err,
        teloxide::RequestError::Api(teloxide::ApiError::BotBlocked)
    ) {
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
        }
    }
}
