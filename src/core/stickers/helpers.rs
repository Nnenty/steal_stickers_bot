use random_string::generate;
use telers::types::Sticker;

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

pub fn generate_sticker_set_name_and_link(length: usize, bot_username: &str) -> (String, String) {
    let charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ1234567890";
    let charset_without_nums = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

    let set_name_prefix = generate(1, charset_without_nums);
    let set_name = generate(length - 1, charset);
    let set_name = format!("{set_name_prefix}{set_name}_by_{bot_username}");

    let set_link = format!("t.me/addstickers/{}", set_name);

    (set_name, set_link)
}
