use teloxide::{adaptors::DefaultParseMode, prelude::*, utils::command::BotCommand};

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "These commands are supported:")]
enum TelegramCommand {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "notify me when there is an update")]
    NotifyMe,
}

async fn answer(
    cx: UpdateWithCx<DefaultParseMode<AutoSend<Bot>>, Message>,
    command: TelegramCommand,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match command {
        TelegramCommand::Help => cx.answer(TelegramCommand::descriptions()).await?,
        TelegramCommand::NotifyMe => {
            cx.answer(format!(
                "Put the following chat id on tanoshi profile settings: {}",
                cx.chat_id()
            ))
            .await?
        }
    };

    Ok(())
}

pub async fn run(name: String, bot: DefaultParseMode<AutoSend<Bot>>) {
    info!("start telegram bot");
    teloxide::commands_repl(
        bot,
        name,
        answer,
    ).await;
}
