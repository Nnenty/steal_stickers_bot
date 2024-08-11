use telers::{
    event::{telegram::HandlerResult, EventReturn},
    fsm::{Context, Storage},
    methods::SendMessage,
    types::Message,
    Bot,
};

use crate::core::send_start_message;

pub async fn start_handler<S: Storage>(
    bot: Bot,
    message: Message,
    fsm: Context<S>,
) -> HandlerResult {
    fsm.finish().await.map_err(Into::into)?;

    bot.send(SendMessage::new(message.chat().id(), send_start_message()))
        .await?;

    Ok(EventReturn::Finish)
}
