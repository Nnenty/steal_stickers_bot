use random_string::generate;
use telers::types::Sticker;

/// Return sticker format for each sticker.
pub fn sticker_format(stickers: &[Sticker]) -> Option<String> {
    stickers.iter().next().map(|sticker| {
        if sticker.is_animated {
            "animated".to_owned()
        } else if sticker.is_video {
            "video".to_owned()
        } else {
            "static".to_owned()
        }
    })
}

/// Generate new random sticker set name. This function assumes that the `length` field is a **positive integer greater than 2**!
/// Otherwise errors may occur using the generated name.
pub fn generate_sticker_set_name_and_link(length: usize, bot_username: &str) -> (String, String) {
    let charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890";
    let charset_without_nums = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

    let set_name_prefix = generate(1, charset_without_nums);
    let set_name = generate(length - 1, charset);
    let set_name = format!("{set_name_prefix}{set_name}_by_{bot_username}");

    let set_link = format!("t.me/addstickers/{}", set_name);

    (set_name, set_link)
}

/// Return begin and end of specify page, using the necessary information
/// (more about the pages, etc. [here](../../bot_commands/handlers/my_stickers.rs)).
pub fn get_page_begin_and_end(
    current_page: usize,
    pages_number: u32,
    list_len: usize,
    sticker_sets_number_per_page: usize,
) -> (usize, usize) {
    let begin_page_index = sticker_sets_number_per_page * (current_page - 1);

    let end_page_index = if current_page == pages_number as usize {
        list_len - 1
    } else {
        begin_page_index + sticker_sets_number_per_page - 1
    };

    (begin_page_index, end_page_index)
}
