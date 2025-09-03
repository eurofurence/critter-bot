use color_eyre::eyre;
use reqwest::StatusCode;
use teloxide::{dispatching::dialogue::GetChatId, prelude::*};
use tracing::error;
use uuid::Uuid;

use crate::State;

async fn default(State { api, bot, db, .. }: State, msg: Message) -> eyre::Result<()> {
    let (Some(chat_id), Some(uname), Some(text)) = (msg.chat_id(), msg.chat.username(), msg.text())
    else {
        return Ok(());
    };
    if db.check_if_present(chat_id).await?.is_some() {
        // No futher action required, the user is already verified
        return Ok(());
    }

    if !text.starts_with("/start ") {
        bot.send_message(chat_id, "Unknown command provided.\nTry logging in via the web interface https://critter.eurofurence.org/").await?;
        return Ok(());
    }
    let token = text[7..].trim();
    let Ok(token) = Uuid::parse_str(token) else {
        bot.send_message(chat_id, "Malformed token provided.\nTry logging in via the web interface https://critter.eurofurence.org/").await?;
        return Ok(());
    };
    let uid = match api.verify(token, uname.into()).await? {
        Ok(uid) => uid,
        Err(reason) => {
            bot.send_message(chat_id, reason).await?;
            return Ok(());
        }
    };
    db.register(uid, chat_id).await?;

    bot.send_message(chat_id, "Your account has been linked successfully!\nFrom now on you will receive notification on any of your upcoming shifts.\nThank you for helping us out!")
        .await?;

    Ok(())
}

async fn spawn_default(state: State, msg: Message) -> eyre::Result<()> {
    tokio::spawn(async move {
        let Err(err) = default(state, msg).await else {
            return;
        };
        error!("Error in bot process occured: {err}");
    });
    Ok(())
}

pub async fn start_bot(state: State) {
    let handler = Update::filter_message().endpoint(spawn_default);

    Dispatcher::builder(state.bot.clone(), handler)
        .dependencies(dptree::deps![state])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;

    std::process::exit(0);
}
