use crate::{api::Api, db::Database};
use chrono_tz::Tz;
use clap::{
    Arg, ArgAction, Command,
    builder::{RangedU64ValueParser, StringValueParser},
};
use color_eyre::eyre;
use reqwest::{
    Client, ClientBuilder, Url,
    header::{AUTHORIZATION, HeaderMap, HeaderValue},
};
use sqlx::PgPool;
use std::{fs, sync::Arc};
use teloxide::Bot;
use tracing_subscriber::EnvFilter;

mod api;
mod bot;
mod db;
mod events;

#[derive(Clone)]
pub struct State {
    api: Api,
    bot: Bot,
    db: Database,
    tz: Tz,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    if fs::exists(".env")? {
        dotenvy::dotenv()?;
    }
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let matches = Command::new("critter-bot")
        .arg(
            Arg::new("pool")
                .env("DATABASE_URL")
                .short('p')
                .long("pool")
                .help("Postgres database url")
                .value_parser(StringValueParser::new())
                .required(true),
        )
        .arg(
            Arg::new("token")
                .env("TELEGRAM_TOKEN")
                .short('t')
                .long("token")
                .help("Token to be use for telegram bot")
                .value_parser(StringValueParser::new())
                .required(true),
        )
        .arg(
            Arg::new("critter-token")
                .env("CRITTER_TOKEN")
                .short('c')
                .long("critter-token")
                .help("Token to be use for talking to the crittersystem")
                .value_parser(StringValueParser::new())
                .required(true),
        )
        .arg(
            Arg::new("critter-baseurl")
                .env("CRITTER_BASEURL")
                .long("critter-baseurl")
                .help("Baseurl of crittersystem: e.g. `https://critter.eurofurence.org/`")
                .default_value("https://critter.eurofurence.org/")
                .value_parser(StringValueParser::new())
                .required(true),
        )
        .arg(
            Arg::new("no-migrate")
                .env("NO_MIGRATE")
                .long("no-migrate")
                .action(ArgAction::SetTrue)
                .help("Prevents migrations from running on bot start, potentially unsafe!")
                .required(true),
        )
        .arg(
            Arg::new("timezone")
                .env("TIMEZONE")
                .long("timezone").
                short('z')
                .help("Sets the events timezone using a TZ identifier code, such as `Europe/Berlin`")
                .default_value("Europe/Berlin")
                .value_parser(clap::value_parser!(Tz))
                .required(true),
        )
        .arg(
            Arg::new("pq-lim")
                .env("PARALLEL_LOOKUP_LIMIT")
                .help("Sets a limit to how many user lookups the tool can make at once in the database")
                .value_parser(RangedU64ValueParser::<usize>::new())
                .default_value("16")
                .required(true),
        )
        .get_matches();

    let pool = PgPool::connect(matches.get_one::<String>("pool").unwrap().as_str()).await?;

    if !matches.get_flag("no-migrate") {
        sqlx::migrate!().run(&pool).await?;
    }

    let token = matches.get_one::<String>("critter-token").unwrap();
    let bot = Bot::new(matches.get_one::<String>("token").unwrap());

    let pq_limit = *matches.get_one::<usize>("pq-lim").unwrap();
    let state = State {
        api: Api::new(matches.get_one::<String>("critter-baseurl").unwrap(), token)?,
        bot,
        db: Database::new(pool, pq_limit),
        tz: *matches.get_one("timezone").unwrap(),
    };

    tokio::spawn(bot::start_bot(state.clone()));
    retry!(events::start_event_processor(state.clone()));

    Ok(())
}

#[macro_export]
macro_rules! retry {
    ($func:expr) => {{
        use std::time::{Duration, Instant};
        use tokio::time::sleep;

        const BASELINE: Duration = Duration::from_secs(1);
        const INC: Duration = Duration::from_secs(5);

        let mut backoff = BASELINE;
        let mut last = Instant::now();

        loop {
            break match $func.await {
                Ok(val) => val,
                Err(err) => {
                    tracing::error!("{err}");
                    if backoff.as_secs() < 30 {
                        backoff += INC;
                    }
                    if last.elapsed().as_secs() > 300 {
                        backoff = BASELINE;
                    }
                    sleep(backoff).await;
                    last = Instant::now();
                    continue;
                }
            };
        }
    }};
}
