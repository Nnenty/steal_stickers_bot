use telers::utils::text::{html_code, html_text_link};

use super::{common::get_page_begin_and_end, constants::STICKER_SETS_NUMBER_PER_PAGE};

pub fn sticker_set_message(
    sticker_set_title: &str,
    sticker_set_name: &str,
    sticker_set_link: &str,
    other_sticker_set_title: &str,
    other_sticker_set_link: &str,
) -> String {
    format!(
        "
        Now you have your own sticker pack {new_ss_url}! \
        You can add stickers to this pack using command /add_stickers! \
        (original: {steal_ss_url})\n\nIf you want to update your new sticker pack, use official Telegram \
        bot @Stickers, which does an excellent job of managing sticker packs. \
        (the name of your new sticker pack to handle it in @Stickers bot: {sticker_set_name})
        ",
        new_ss_url = html_text_link(sticker_set_title, sticker_set_link),
        steal_ss_url = html_text_link(other_sticker_set_title, other_sticker_set_link,),
        sticker_set_name = html_code(sticker_set_name)
    )
}

pub fn start_message() -> String {
    "
    Hello! This is bot to steal stickers!\n\
    List of commands you can use:\n\
    /help - Show help message\n\
    /source or /src - Show source code of the bot\n\
    /cancel - Cancel last command\n\
    /steal_pack - Steal sticker pack\n\
    /add_stickers - Add sticker to a sticker pack stolen by this bot\n\
    /my_stickers - List of your stolen stickers\n\
        "
    .to_owned()
}

pub fn current_page_message(current_page: usize, pages_number: u32, list: &Vec<String>) -> String {
    let (begin_page_index, end_page_index) = get_page_begin_and_end(
        current_page,
        pages_number,
        list.len(),
        STICKER_SETS_NUMBER_PER_PAGE,
    );

    let mut sticker_sets_page =
        String::from(format!("List of your stickers ({current_page} page):\n"));
    for i in begin_page_index..end_page_index {
        sticker_sets_page += list[i].as_str();
        sticker_sets_page.push(' ');
    }

    sticker_sets_page
}

#[test]
fn current_page_message_test() {
    let mut list = Vec::new();
    for i in 0..78 {
        list.push(format!("set{i}"));
    }

    let message = current_page_message(2, 2, &list);

    assert_eq!(
        message.as_str(),
        "List of your stickers (2 page):\n\
    set50 set51 set52 set53 set54 set55 set56 set57 set58 set59 set60 set61 set62 set63 set64 \
    set65 set66 set67 set68 set69 set70 set71 set72 set73 set74 set75 set76 set77 \
    "
    );
}
