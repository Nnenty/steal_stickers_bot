use telers::event::telegram::HandlerResult;
use telers::event::EventReturn;
use telers::methods::{SendMessage, SetMyCommands};
use telers::types::Message;
use telers::Bot;

pub async fn start_handler(bot: Bot, message: Message) -> HandlerResult {
    bot.send(SendMessage::new(
        message.chat().id(),
        "
    Hello! This is bot to steal stickers.\n\
    List of commands you can use:
/help - Show help message
/steal - Steal sticker pack
/my_stickers - Your current sticker packs you was steal
        ",
    ))
    .await?;

    Ok(EventReturn::Finish)
}
