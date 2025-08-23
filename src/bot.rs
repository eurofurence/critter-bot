use color_eyre::eyre;
use teloxide::prelude::*;
use tracing::debug;

pub async fn run_bot(token: impl Into<String>) -> eyre::Result<()> {
    let bot = Bot::new(token);

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
        debug!("{:?}", msg.text());
        Ok(())
    })
    .await;

    Ok(())
}
