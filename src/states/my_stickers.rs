use std::borrow::Cow;

#[derive(Clone)]
pub enum MyStickersState {
    GetStolenStickerSet,
    GetStickersToAdd,
}

impl MyStickersState {
    const fn as_str(&self) -> &'static str {
        match self {
            MyStickersState::GetStolenStickerSet => "get_stolen_sticker_set",
            MyStickersState::GetStickersToAdd => "get_stickers_to_add",
        }
    }
}

impl From<MyStickersState> for Cow<'static, str> {
    fn from(state: MyStickersState) -> Self {
        Cow::Borrowed(state.as_str())
    }
}

impl PartialEq<&str> for MyStickersState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
