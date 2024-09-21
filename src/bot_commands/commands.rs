use crate::bot_commands::states::{AddStickerState, MyStickersState, StealStickerSetState};
use telers::{
    client::Reqwest,
    enums::{ChatType as ChatTypeEnum, ContentType as ContentTypeEnum},
    filters::{ChatType, Command, ContentType, State as StateFilter},
    fsm::MemoryStorage,
    Filter as _, Router,
};

use super::handlers::{
    add_stickers, add_stickers_to_user_owned_sticker_set, cancel, create_new_sticker_set,
    get_stickers_to_add, get_stolen_sticker_set, my_stickers as my_stickers_handler,
    process_button, process_non_sticker as process_non_sticker_handler, source, start,
    steal_sticker_set, steal_sticker_set_name,
};

/// If the user simply writes to the bot without calling any commands, the bot will call specified function
pub async fn process_non_command(router: &mut Router<Reqwest>, ignore_commands: &'static [&str]) {
    router
        .message
        .register(start::<MemoryStorage>)
        .filter(StateFilter::none())
        .filter(Command::many(ignore_commands.iter().map(ToOwned::to_owned)).invert());
}

/// Executes Telegram commands `/start` and `/help`
pub async fn start_command(router: &mut Router<Reqwest>, commands: &'static [&str]) {
    router
        .message
        .register(start::<MemoryStorage>)
        .filter(ChatType::one(ChatTypeEnum::Private))
        .filter(Command::many(commands.iter().map(ToOwned::to_owned)));
}

/// Executes Telegram commands `/src` and `/source`
pub async fn source_command(router: &mut Router<Reqwest>, commands: &'static [&str]) {
    router
        .message
        .register(source::<MemoryStorage>)
        .filter(ChatType::one(ChatTypeEnum::Private))
        .filter(Command::many(commands.iter().map(ToOwned::to_owned)));
}

/// Executes Telegram command `/cancel`
pub async fn cancel_command(router: &mut Router<Reqwest>, commands: &'static [&str]) {
    router
        .message
        .register(cancel::<MemoryStorage>)
        .filter(ChatType::one(ChatTypeEnum::Private))
        .filter(Command::many(commands.iter().map(ToOwned::to_owned)));
}

/// Executes Telegram command `/add_stickers`
pub async fn add_stickers_command(
    router: &mut Router<Reqwest>,
    command: &'static str,
    done_command: &'static str,
) {
    router
        .message
        .register(add_stickers::<MemoryStorage>)
        .filter(ChatType::one(ChatTypeEnum::Private))
        .filter(Command::one(command))
        .filter(ContentType::one(ContentTypeEnum::Text));

    router
        .message
        .register(get_stolen_sticker_set::<MemoryStorage>)
        .filter(ContentType::one(ContentTypeEnum::Sticker))
        .filter(StateFilter::one(AddStickerState::GetStolenStickerSet));

    router
        .message
        .register(get_stickers_to_add::<MemoryStorage>)
        .filter(ContentType::one(ContentTypeEnum::Sticker))
        .filter(StateFilter::one(AddStickerState::GetStickersToAdd));

    router
        .message
        .register(add_stickers_to_user_owned_sticker_set::<MemoryStorage>)
        .filter(Command::one(done_command))
        .filter(ContentType::one(ContentTypeEnum::Text))
        .filter(StateFilter::one(AddStickerState::GetStickersToAdd));
}

/// Executes Telegram command `/steal_pack`
pub async fn steal_sticker_set_command(router: &mut Router<Reqwest>, command: &'static str) {
    router
        .message
        .register(steal_sticker_set::<MemoryStorage>)
        .filter(ChatType::one(ChatTypeEnum::Private))
        .filter(Command::one(command))
        .filter(ContentType::one(ContentTypeEnum::Text));

    router
        .message
        .register(steal_sticker_set_name::<MemoryStorage>)
        .filter(ContentType::one(ContentTypeEnum::Sticker))
        .filter(StateFilter::one(StealStickerSetState::StealStickerSetName));

    router
        .message
        .register(create_new_sticker_set::<MemoryStorage>)
        .filter(ContentType::one(ContentTypeEnum::Text))
        .filter(StateFilter::one(StealStickerSetState::CreateNewStickerSet));
}

pub async fn my_stickers(router: &mut Router<Reqwest>, command: &'static str) {
    router
        .message
        .register(my_stickers_handler::<MemoryStorage>)
        .filter(Command::one(command))
        .filter(ContentType::one(ContentTypeEnum::Text));

    router
        .callback_query
        .register(process_button::<MemoryStorage>)
        .filter(StateFilter::one(
            MyStickersState::EditStickerSetsListMessage,
        ));
}

/// If user enter wrong content type, but the request type is <content_type>, this handler will process it
pub async fn process_non_sticker(router: &mut Router<Reqwest>, content_type: ContentTypeEnum) {
    router
        .message
        .register(process_non_sticker_handler)
        .filter(ContentType::one(content_type).invert())
        .filter(
            StateFilter::one(StealStickerSetState::StealStickerSetName).or(StateFilter::many([
                AddStickerState::GetStolenStickerSet,
                AddStickerState::GetStickersToAdd,
            ])),
        );
}
