use telers::{
    event::{telegram::HandlerResult, EventReturn},
    methods::SendMessage,
    types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, MessageText, ReplyMarkup},
    Bot,
};
use tracing::debug;

const STICKERS_NUMBER_PER_PAGE: usize = 10;

pub async fn my_stickers(bot: Bot, message: MessageText) -> HandlerResult {
    // In the future, a database will be added, and the real response from the database will be used instead of this
    // variable. So far, during the development of the algorithm itself, I use a stub
    let mut database_result = Vec::new();
    for i in 0..38 {
        database_result.push(format!("set{i}"));
    }

    let mut buttons = Vec::new();

    if database_result.len() > STICKERS_NUMBER_PER_PAGE {
        database_result.iter().enumerate().for_each(|(i, _)| {
            if i % STICKERS_NUMBER_PER_PAGE == 0 {
                buttons.push(
                    InlineKeyboardButton::new(format!("page {i}"))
                        .callback_data(format!("page_{i}")),
                )
            }
        })
    } else if database_result.len() > 0 {
        buttons.push(InlineKeyboardButton::new("page 1").callback_data("page_1"));
    } else {
        bot.send(SendMessage::new(
            message.chat.id(),
            "You don't have a single stolen sticker pack! \
        Steal any sticker pack using the /steal_pack command and it will appear in this list!",
        ))
        .await?;

        return Ok(EventReturn::Finish);
    };

    let inline_keyboard = ReplyMarkup::InlineKeyboard(InlineKeyboardMarkup::new([buttons]));

    bot.send(
        SendMessage::new(message.chat.id(), "List of your stolen stickers:")
            .reply_markup(inline_keyboard),
    )
    .await?;

    Ok(EventReturn::Finish)
}

pub async fn process_button(bot: Bot, message: CallbackQuery) -> HandlerResult {
    bot.send(SendMessage::new(
        message.chat_id().unwrap(),
        message.data.unwrap(),
    ))
    .await?;

    Ok(EventReturn::Finish)
}
