use telers::{
    event::{telegram::HandlerResult, EventReturn},
    methods::SendMessage,
    types::{CallbackQuery, InlineKeyboardButton, InlineKeyboardMarkup, MessageText, ReplyMarkup},
    Bot,
};
use tracing::debug;

const STICKER_SETS_NUMBER_PER_PAGE: usize = 10;

pub async fn my_stickers(bot: Bot, message: MessageText) -> HandlerResult {
    // In the future, a database will be added, and the real response from the database will be used instead of this
    // variable. So far, during the development of the algorithm itself, I use a stub
    let mut database_result = Vec::new();
    for i in 0..500 {
        database_result.push(format!("set{i}"));
    }

    let mut buttons = Vec::new();

    let mut page_count = 0;
    let mut current_row_index = 0;

    if database_result.len() > STICKER_SETS_NUMBER_PER_PAGE {
        database_result.iter().enumerate().for_each(|(i, _)| {
            if i % STICKER_SETS_NUMBER_PER_PAGE == 0 {
                // create a new row every 5 buttons
                if page_count % 5 == 0 {
                    page_count += 1;
                    current_row_index += 1;

                    buttons.push(vec![InlineKeyboardButton::new(format!(
                        "page {page_count}",
                    ))
                    .callback_data(format!("page_{page_count}",))])
                // else push button into current row
                } else {
                    page_count += 1;

                    buttons[current_row_index - 1].push(
                        InlineKeyboardButton::new(format!("page {page_count}",))
                            .callback_data(format!("page_{page_count}",)),
                    );
                }
            }
        })
    } else if database_result.len() > 0 {
        buttons.push(vec![
            InlineKeyboardButton::new("page 1").callback_data("page_1")
        ]);
    } else {
        bot.send(SendMessage::new(
            message.chat.id(),
            "You don't have a single stolen sticker pack! \
        Steal any sticker pack using the /steal_pack command and it will appear in this list!",
        ))
        .await?;

        return Ok(EventReturn::Finish);
    };

    let inline_keyboard = ReplyMarkup::InlineKeyboard(InlineKeyboardMarkup::new(buttons));

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
