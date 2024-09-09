use telers::{
    event::{telegram::HandlerResult, EventReturn},
    fsm::{Context, Storage},
    methods::{EditMessageText, SendMessage},
    types::{
        CallbackQuery, ChatIdKind, InlineKeyboardButton, InlineKeyboardMarkup, Message,
        MessageText, ReplyMarkup,
    },
    Bot,
};
use tracing::error;

use crate::{states::MyStickersState, texts::current_page_message};

pub const STICKER_SETS_NUMBER_PER_PAGE: usize = 50;

pub async fn my_stickers<S: Storage>(
    bot: Bot,
    message: MessageText,
    fsm: Context<S>,
) -> HandlerResult {
    fsm.finish().await.map_err(Into::into)?;

    // In the future, a database will be added, and the real response from the database will be used instead of this
    // variable. So far, during the development of the algorithm itself, I use a stub
    let mut database_result = Vec::new();
    for i in 0..515 {
        database_result.push(format!("set{}", i + 1));
    }

    let mut buttons = Vec::new();

    let mut page_count: u32 = 0;
    let mut current_row_index = 0;

    if database_result.len() > STICKER_SETS_NUMBER_PER_PAGE || database_result.len() > 0 {
        database_result
            .iter()
            .enumerate()
            .filter(|(index, _)| index % STICKER_SETS_NUMBER_PER_PAGE == 0)
            .for_each(|_| {
                // create a new row every 5 buttons
                if page_count % 5 == 0 {
                    page_count += 1;
                    current_row_index += 1;

                    buttons.push(vec![InlineKeyboardButton::new(format!(
                        "page {page_count}",
                    ))
                    .callback_data(format!("{page_count}",))])
                // else push button into current row
                } else {
                    page_count += 1;

                    buttons[current_row_index - 1].push(
                        InlineKeyboardButton::new(format!("Page {page_count}",))
                            .callback_data(format!("{page_count}",)),
                    );
                }
            })
    // otherwise user does not have sticker sets stolen by this bot
    } else {
        bot.send(SendMessage::new(
            message.chat.id(),
            "You don't have a single stolen sticker pack! \
        Steal any sticker pack using the /steal_pack command and it will appear in this list!",
        ))
        .await?;

        return Ok(EventReturn::Finish);
    };

    let inline_keyboard_markup = InlineKeyboardMarkup::new(buttons);
    let inline_keyboard = ReplyMarkup::InlineKeyboard(inline_keyboard_markup.clone());

    let sticker_sets_list_message = bot
        .send(
            SendMessage::new(
                message.chat.id(),
                current_page_message(1, page_count, &database_result),
            )
            .reply_markup(inline_keyboard),
        )
        .await?;

    fsm.set_value("edit_sticker_sets_list_message", sticker_sets_list_message)
        .await
        .map_err(Into::into)?;

    fsm.set_value(
        "sticker_sets_list_inline_keyboard_markup",
        inline_keyboard_markup,
    )
    .await
    .map_err(Into::into)?;

    fsm.set_value("pages_number", page_count)
        .await
        .map_err(Into::into)?;

    fsm.set_state(MyStickersState::EditStickerSetsListMessage)
        .await
        .map_err(Into::into)?;

    Ok(EventReturn::Finish)
}

pub async fn process_button<S: Storage>(
    bot: Bot,
    message: CallbackQuery,
    fsm: Context<S>,
) -> HandlerResult {
    let message_data = match message.data {
        Some(message_data) => message_data,
        None => {
            error!(
                "None value occurded while processed callback query from inline keyboard button!"
            );

            bot.send(SendMessage::new(
                message.chat_id().expect("chat not found"),
                "Sorry, an error occurs :( Try again.",
            ))
            .await?;

            return Ok(EventReturn::Finish);
        }
    };

    // process if user click to one button several times
    match fsm
        .get_value::<_, Box<str>>("previous_callback_query")
        .await
        .map_err(Into::into)?
    {
        Some(msg_data) => {
            if msg_data == message_data {
                // do nothing
                return Ok(EventReturn::Finish);
            } else {
                fsm.set_value("previous_callback_query", Some(message_data.as_ref()))
                    .await
                    .map_err(Into::into)?;
            }
        }
        None => {
            fsm.set_value("previous_callback_query", Some(message_data.as_ref()))
                .await
                .map_err(Into::into)?;
        }
    };

    // In the future, a database will be added, and the real response from the database will be used instead of this
    // variable. So far, during the development of the algorithm itself, I use a stub
    let mut database_result = Vec::new();
    for i in 0..515 {
        database_result.push(format!("set{}", i + 1));
    }

    let pages_number: u32 = fsm
        .get_value("pages_number")
        .await
        .map_err(Into::into)?
        .expect("Number of pages should be set");

    let current_page = message_data
        .parse::<usize>()
        .expect("fail to convert `message_data` string into usize");

    let sticker_sets_page = current_page_message(current_page, pages_number, &database_result);

    let message_to_edit: Message = fsm
        .get_value("edit_sticker_sets_list_message")
        .await
        .map_err(Into::into)?
        .expect("Sticker sets list message should be set");

    let message_to_edit_reply_markup: InlineKeyboardMarkup = fsm
        .get_value("sticker_sets_list_inline_keyboard_markup")
        .await
        .map_err(Into::into)?
        .expect("Inline keyboard for sticker sets list should be set");

    let message_to_edit_chat_id = ChatIdKind::id(message_to_edit.chat().id());
    let message_to_edit_id = message_to_edit.id();

    let edit_message = EditMessageText::new(sticker_sets_page)
        .chat_id(message_to_edit_chat_id)
        .message_id(message_to_edit_id)
        .reply_markup(message_to_edit_reply_markup);

    bot.send(edit_message).await?;

    Ok(EventReturn::Finish)
}
