use std::fs;

use clap::{
    Arg, ArgAction, Command,
    builder::{BoolValueParser, StringValueParser},
};
use color_eyre::eyre;
use sqlx::PgPool;
use tracing_subscriber::EnvFilter;

mod bot;

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
                .help("Token to use for telegram bot")
                .value_parser(StringValueParser::new())
                .required(true),
        )
        .arg(
            Arg::new("no-migrate")
                .env("NO_MIGRATE")
                .long("no-migrate")
                .action(ArgAction::SetTrue)
                .help("Prevents migrations from running on bot start, potentially unsafe!"),
        )
        .get_matches();

    let pool = PgPool::connect(matches.get_one::<String>("pool").unwrap().as_str()).await?;

    if !matches.get_flag("no-migrate") {
        sqlx::migrate!().run(&pool).await?;
    }

    bot::run_bot(matches.get_one::<String>("token").unwrap()).await?;

    Ok(())
}
