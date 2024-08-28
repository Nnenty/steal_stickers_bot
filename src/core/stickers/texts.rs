use telers::utils::text::{html_code, html_text_link};

pub fn send_sticker_set_message(
    sticker_set_title: &str,
    sticker_set_name: &str,
    sticker_set_link: &str,
    other_sticker_set_title: &str,
    other_sticker_set_link: &str,
) -> String {
    format!(
        "
        Now you have your own sticker pack {new_ss_url}!\n\
        You can add stickers to this pack using command /add_stickers!\n\
        Don't forget to add them for yourself!\n(original {steal_ss_url})\n\nIf you want to update your \
        new sticker pack, use official Telegram bot @Stickers, which does an excellent job of managing sticker packs.\n\
        (the name of your new sticker pack to handle it in @Stickers:\n{sticker_set_name})
        ",
        new_ss_url = html_text_link(sticker_set_title, sticker_set_link),
        steal_ss_url = html_text_link(other_sticker_set_title, other_sticker_set_link,),
        sticker_set_name = html_code(sticker_set_name)
    )
}

pub fn send_start_message() -> String {
    "
    Hello! This is bot to steal stickers.\n\
    List of commands you can use:\n\
    /help - Show help message\n\
    /source or /src - Show source code of the bot\n\
    /steal_pack - Steal sticker pack\n\
    /add_stickers - Add sticker to a sticker pack stolen by this bot\n\
    /cancel - Cancel last command
        "
    .to_owned()
}
