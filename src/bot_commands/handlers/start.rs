use telers::{
    event::{telegram::HandlerResult, EventReturn},
    fsm::{Context, Storage},
    methods::SendMessage,
    types::Message,
    Bot,
};

use crate::texts::start_message;

pub async fn start_handler<S: Storage>(
    bot: Bot,
    message: Message,
    fsm: Context<S>,
) -> HandlerResult {
    fsm.finish().await.map_err(Into::into)?;

    // only can panic if messages uses in channels, but i'm using private filter in main function
    let user_first_name = &message.from().expect("error while parsing user").first_name;

    bot.send(SendMessage::new(
        message.chat().id(),
        start_message(user_first_name),
    ))
    .await?;

    Ok(EventReturn::Finish)
}
