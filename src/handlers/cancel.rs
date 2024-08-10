use telers::{
    event::{telegram::HandlerResult, EventReturn},
    fsm::{Context, Storage},
    methods::SendMessage,
    types::MessageText,
    Bot,
};

pub async fn cancel_handler<S: Storage>(
    bot: Bot,
    message: MessageText,
    fsm: Context<S>,
) -> HandlerResult {
    fsm.finish().await.map_err(Into::into)?;

    bot.send(SendMessage::new(
        message.chat.id(),
        "Last command was canceled.",
    ))
    .await?;

    Ok(EventReturn::Cancel)
}
