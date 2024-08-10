use random_string::generate;
use telers::types::Sticker;

pub fn sticker_format(stickers: &[Sticker]) -> Option<&str> {
    stickers.iter().next().map(|sticker| {
        if sticker.is_animated {
            "animated"
        } else if sticker.is_video {
            "video"
        } else {
            "static"
        }
    })
}

pub fn generate_sticker_set_name_and_link(length: usize, bot_username: &str) -> (String, String) {
    let charset = "abcg890hijklmJKxyzAnopqrstuvwBefCDEFGHIQRSTUVWXYZ1237LMNOP45d6";

    let set_name = String::from(generate(length, charset));
    let set_name = format!("{}_by_{}", set_name, bot_username);

    let set_link = format!("t.me/addstickers/{}", set_name);

    (set_name, set_link)
}
