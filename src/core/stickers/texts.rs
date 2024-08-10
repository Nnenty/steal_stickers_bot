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
        Then you have your own sticker pack {new_ss_url}!\n(original {steal_ss_url})\n\nIf you want to update your \
        new sticker pack, use official Telegram bot @Stickers, which does an excellent job of managing sticker packs.\n\
        (the name of your new sticker pack to handle it in @Stickers:\n{sticker_set_name})
\n\
        List of commands in @Stickers that may be useful to you:\n\
        /addsticker – add a sticker to an existing set\n\
        /editsticker – change emoji or coordinates\n\
        /replacesticker – replace stickers in a set\n\
        /ordersticker – reorder stickers in a set\n\
        /delsticker – remove a sticker from an existing set\n\
        /setpackicon – set a sticker set icon\n\
        /renamepack – rename a set\n\
        /delpack – delete a set
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
    /steal - Steal sticker pack\n\
    /cancel - Cancel last command
        "
    .to_owned()
}
