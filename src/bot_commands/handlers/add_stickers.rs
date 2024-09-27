use std::time::Duration;

use telers::{
    enums::ParseMode,
    errors::HandlerError,
    event::{telegram::HandlerResult, EventReturn},
    fsm::{Context, Storage},
    methods::{DeleteMessage, GetMe, GetStickerSet, SendMessage},
    types::{MessageSticker, MessageText, ReplyParameters, Sticker},
    utils::text::{html_bold, html_text_link},
    Bot,
};

use tracing::error;

use crate::{
    application::{
        common::{
            exceptions::{RepoKind, TransactionKind},
            traits::uow::{UoW as UoWTrait, UoWFactory as UoWFactoryTrait},
        },
        set::{dto::create::Create as CreateSet, traits::SetRepo},
        user::{
            dto::{create::Create as CreateUser, get_by_tg_id::GetByTgID as GetUserByTgID},
            traits::UserRepo,
        },
    },
    bot_commands::{handlers::add_stickers, states::AddStickerState},
    core::stickers::constants::MAX_STICKER_SET_LENGTH,
    middlewares::client_application::Client,
    telegram_application::{get_owned_stolen_sticker_sets, get_sticker_set_user_id},
};

pub async fn add_stickers_handler<S: Storage>(
    bot: Bot,
    message: MessageText,
    fsm: Context<S>,
) -> HandlerResult {
    fsm.finish().await.map_err(Into::into)?;

    fsm.set_state(AddStickerState::GetStolenStickerSet)
        .await
        .map_err(Into::into)?;

    bot.send(SendMessage::new(
            message.chat.id(),
            format!("Send me {your} sticker pack, in which you want to add stickers. You can see all your \
            stolen stickers, using command /mystickers (if you don't have the sticker packs stolen by this bot, first use the command /stealpack).",
            your = html_bold("your stolen")),
        ).parse_mode(ParseMode::HTML))
        .await?;

    Ok(EventReturn::Finish)
}

pub async fn get_stolen_sticker_set<S, UoWFactory>(
    bot: Bot,
    message: MessageSticker,
    fsm: Context<S>,
    Client(client): Client,
    uow_factory: UoWFactory,
) -> HandlerResult
where
    UoWFactory: UoWFactoryTrait,
    S: Storage,
{
    let sticker_set_name = match message.sticker.set_name {
        Some(sticker_set_name) => sticker_set_name,
        None => {
            bot.send(SendMessage::new(
                message.chat.id(),
                "This sticker is without sticker pack! Try to send another sticker pack.",
            ))
            .await?;

            return Ok(EventReturn::Finish);
        }
    };

    let sticker_set = bot
        .send(GetStickerSet::new(sticker_set_name.as_ref()))
        .await?;

    let sticker_set_title = sticker_set.title;

    // cant panic
    let bot_username = bot
        .send(GetMe::new())
        .await?
        .username
        .expect("bot without username :/");

    if !sticker_set
        .name
        .ends_with(format!("by_{bot_username}").as_str())
    {
        bot.send(SendMessage::new(
            message.chat.id(),
            format!(
            "This sticker pack wasnt stolen by this bot, which means i cant add stickers to it according to Telegram rules! \
            Please, send {your} sticker pack or steal this sticker pack using command /stealpack.",
            your = html_bold("your stolen")
            )
        ).parse_mode(ParseMode::HTML))
        .await?;

        return Ok(EventReturn::Finish);
    }

    // if function doesnt execute in 3 second, send error message
    let user_id = match tokio::time::timeout(
        Duration::from_secs(3),
        get_sticker_set_user_id(sticker_set_name.as_ref(), &client),
    )
    .await
    {
        Ok(Ok(set_id)) => set_id,
        Ok(Err(err)) => {
            error!(%err, "failed to get sticker set user id:");

            bot.send(
                SendMessage::new(
                    message.chat.id(),
                    "Sorry, an error occurs. Try send this sticker again :(",
                )
                .reply_parameters(ReplyParameters::new(message.id).chat_id(message.chat.id())),
            )
            .await?;

            return Ok(EventReturn::Finish);
        }
        Err(err) => {
            error!(%err, "too long time to get sticker set user id:");

            bot.send(
                SendMessage::new(
                    message.chat.id(),
                    "Sorry, an error occurs. Try send sticker again :(",
                )
                .reply_parameters(ReplyParameters::new(message.id).chat_id(message.chat.id())),
            )
            .await?;

            return Ok(EventReturn::Finish);
        }
    };

    let mut uow = uow_factory.create_uow();

    create_sticker_set_or_ignore(
        &mut uow,
        sticker_set_name.as_ref(),
        sticker_set_title.as_ref(),
        user_id,
    )
    .await
    .map_err(HandlerError::new)?;

    create_user_or_ignore(&mut uow, user_id)
        .await
        .map_err(HandlerError::new)?;

    // if let Err(err) = get_owned_stolen_sticker_sets(&client, user_id, &bot_username).await {
    //     error!(%err, "failed to get user owned stolen sticker sets:");

    //     bot.send(SendMessage::new(
    //         message.chat.id(),
    //         "Sorry, an error occurs. Try send this sticker again :(",
    //     ))
    //     .await?;

    //     return Ok(EventReturn::Finish);
    // }

    // only panic if messages uses in channels, but i'm using private filter in main function
    let sender_user_id = message.from.expect("user not specified").id;

    if sender_user_id != user_id {
        bot.send(
            SendMessage::new(
                message.chat.id(),
                format!(
                    "You are not the owner of this sticker pack! Please, send {your} sticker pack \
            or steal this sticker pack using command /stealpack.",
                    your = html_bold("your stolen")
                ),
            )
            .parse_mode(ParseMode::HTML),
        )
        .await?;

        return Ok(EventReturn::Finish);
    }

    let set_length = bot
        .send(GetStickerSet::new(sticker_set_name.as_ref()))
        .await?
        .stickers
        .len();

    let message_delete = if MAX_STICKER_SET_LENGTH - set_length > 0 {
        bot.send(SendMessage::new(
                message.chat.id(),
                format!("Total length of this sticker pack = {set_length}. This means you can add a maximum of {} stickers, \
                otherwise you will get error because the maximum size of a sticker pack in current time = {MAX_STICKER_SET_LENGTH} stickers.",
                MAX_STICKER_SET_LENGTH - set_length),
            ).reply_parameters(ReplyParameters::new(message.id).chat_id(message.chat.id())))
            .await?
    } else {
        bot.send(SendMessage::new(
                message.chat.id(),
                format!("Sorry, but this sticker pack contains {MAX_STICKER_SET_LENGTH} stickers! :(\n\
                You cant add more stickers, because the maximum size of a sticker pack in current time = {MAX_STICKER_SET_LENGTH} \
                stickers. Try send another pack(or delete some stickers from this sticker pack).")
            ).reply_parameters(ReplyParameters::new(message.id).chat_id(message.chat.id())))
            .await?;

        return Ok(EventReturn::Finish);
    };

    fsm.set_value(
        "get_stolen_sticker_set",
        (sticker_set_name, sticker_set_title),
    )
    .await
    .map_err(Into::into)?;

    fsm.set_state(AddStickerState::GetStickersToAdd)
        .await
        .map_err(Into::into)?;

    bot.send(SendMessage::new(
        message.chat.id(),
        "Now send me stickers you want to add in stolen sticker pack. \
        When youre ready, use /done command (or /cancel, if you want to cancel the last command).",
    ))
    .await?;

    // delete unnecessary message after 15 sec
    tokio::time::sleep(Duration::from_secs(15)).await;
    bot.send(DeleteMessage::new(
        message_delete.chat().id(),
        message_delete.id(),
    ))
    .await?;

    Ok(EventReturn::Finish)
}

pub async fn get_stickers_to_add<S: Storage>(
    bot: Bot,
    message: MessageSticker,
    fsm: Context<S>,
) -> HandlerResult {
    let sticker = message.sticker;

    if sticker.emoji.is_none() {
        bot.send(
            SendMessage::new(
                message.chat.id(),
                "Sorry, but this sticker is without emoji. Try send another sticker.",
            )
            .reply_parameters(ReplyParameters::new(message.id).chat_id(message.chat.id())),
        )
        .await?;
    }

    let sticker_vec: Vec<Sticker> = match fsm
        .get_value::<&str, Vec<Sticker>>("get_stickers_to_add")
        .await
        .map_err(Into::into)?
    {
        Some(mut get_sticker_vec) => {
            let (sticker_set_name, _): (Box<str>, Box<str>) = fsm
                .get_value("get_stolen_sticker_set")
                .await
                .map_err(Into::into)?
                // in functions above checks whether a set of stickers has a name
                .expect("Sticker set name for sticker set should be set");

            let set_length = bot
                .send(GetStickerSet::new(sticker_set_name.as_ref()))
                .await?
                .stickers
                .len();

            let sticker_vec_len = get_sticker_vec.len();

            if set_length + sticker_vec_len >= MAX_STICKER_SET_LENGTH {
                bot.send(SendMessage::new(
                    message.chat.id(),
                    format!("Please, use command /done to add stickers (or /cancel if for some reason you change your \
                    mind about adding them), because the sum of the current stickers in the sticker pack \
                    and the stickers you want to add to it has reached {MAX_STICKER_SET_LENGTH}! All next stickers (if you continue sending) \
                    will be ignored!"),
                ))
                .await?;

                return Ok(EventReturn::Finish);
            }

            get_sticker_vec.insert(get_sticker_vec.len(), sticker);

            get_sticker_vec
        }
        None => vec![sticker],
    };

    fsm.set_value("get_stickers_to_add", sticker_vec)
        .await
        .map_err(Into::into)?;

    bot.send(
        SendMessage::new(
            message.chat.id(),
            "Sticker processed! Send the next one, or use the /done command if you're ready.",
        )
        .reply_parameters(ReplyParameters::new(message.id).chat_id(message.chat.id())),
    )
    .await?;

    Ok(EventReturn::Finish)
}

/// ### Panics
/// - Panics if user is unknown (only if message sent in channel)
pub async fn add_stickers_to_user_owned_sticker_set<S: Storage>(
    bot: Bot,
    message: MessageText,
    fsm: Context<S>,
) -> HandlerResult {
    let (sticker_set_name, sticker_set_title): (Box<str>, Box<str>) = fsm
        .get_value("get_stolen_sticker_set")
        .await
        .map_err(Into::into)?
        // only panic if i'm forget call fsm.set_value() in function get_stolen_sticker_set()
        .expect("Sticker set name for sticker set should be set");

    let stickers_to_add_vec: Vec<Sticker> = match fsm
        .get_value("get_stickers_to_add")
        .await
        .map_err(Into::into)?
    {
        Some(sticker_vec) => sticker_vec,
        None => {
            bot.send(SendMessage::new(
                message.chat.id(),
                "You haven't sent a single sticker! Send the stickers, and only then use the /done command.",
            ))
            .await?;

            return Ok(EventReturn::Finish);
        }
    };

    fsm.finish().await.map_err(Into::into)?;

    // only panic if messages uses in channels, but i'm using private filter in main function
    let user_id = message.from.expect("error while parsing user").id;

    let message_delete = bot
        .send(SendMessage::new(
            message.chat.id(),
            "Done! Trying to add that sticker(s) to your sticker pack..\n\
        (if you have sent a lot of stickers, it may take up to a few minutes to add them)",
        ))
        .await?;

    let all_stickers_was_added = add_stickers(
        &bot,
        user_id,
        sticker_set_name.as_ref(),
        stickers_to_add_vec.as_ref(),
    )
    .await
    .expect("empty stickers list");

    if !all_stickers_was_added {
        bot.send(
            SendMessage::new(
                message.chat.id(),
                format!(
                    "Error occurded while adding stickers, {but_added}! Due to an error, not all stickers have been added :(",
                    but_added = html_bold("but some stickers was added"),
                ),
            )
            .parse_mode(ParseMode::HTML),
        )
        .await?;
    }

    bot.send(
        SendMessage::new(
            message.chat.id(),
            format!(
                "This sticker(s) was added into {set}!",
                set = html_text_link(
                    sticker_set_title,
                    format!("t.me/addstickers/{}", sticker_set_name)
                )
            ),
        )
        .parse_mode(ParseMode::HTML),
    )
    .await?;

    // delete unnecessary message
    bot.send(DeleteMessage::new(
        message_delete.chat().id(),
        message_delete.id(),
    ))
    .await?;

    Ok(EventReturn::Finish)
}

pub async fn create_sticker_set_or_ignore<UoW>(
    uow: &mut UoW,
    set_name: &str,
    title: &str,
    tg_id: i64,
) -> Result<(), TransactionKind>
where
    UoW: UoWTrait,
{
    let result = uow
        .set_repo()
        .await
        .map_err(TransactionKind::begin_err)?
        .create(CreateSet::new(tg_id, set_name, title))
        .await;

    match result {
        Ok(_) => (),
        Err(RepoKind::Unexpected(_)) => {
            uow.rollback()
                .await
                .map_err(TransactionKind::rollback_err)?;
        }
        Err(RepoKind::Exception(_)) => {
            return Ok(());
        }
    };

    uow.commit().await.map_err(TransactionKind::commit_err)?;

    Ok(())
}

pub async fn create_user_or_ignore<UoW>(uow: &mut UoW, tg_id: i64) -> Result<(), TransactionKind>
where
    UoW: UoWTrait,
{
    let result = uow
        .user_repo()
        .await
        .map_err(TransactionKind::begin_err)?
        .create(CreateUser::new(tg_id))
        .await;

    match result {
        Ok(_) => (),
        Err(RepoKind::Unexpected(_)) => {
            uow.rollback()
                .await
                .map_err(TransactionKind::rollback_err)?;
        }
        Err(RepoKind::Exception(_)) => {
            return Ok(());
        }
    }

    uow.commit().await.map_err(TransactionKind::commit_err)?;

    Ok(())
}
