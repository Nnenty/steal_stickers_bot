use telers::{
    event::{telegram::HandlerResult, EventReturn},
    fsm::{Context, Storage},
    methods::SendMessage,
    types::MessageText,
    Bot,
};

use crate::core::send_start_message;

pub async fn start_handler<S: Storage>(
    bot: Bot,
    message: MessageText,
    fsm: Context<S>,
) -> HandlerResult {
    bot.send(SendMessage::new(message.chat.id(), send_start_message()))
        .await?;

    fsm.finish().await.map_err(Into::into)?;

    Ok(EventReturn::Finish)
}
