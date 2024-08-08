use std::borrow::Cow;

#[derive(Clone)]
pub enum State {
    StealStickerSetName,
    NewStickerSetName,
    NewStickerSetTitle,
    CreateNewStickerSet,
}

impl State {
    const fn as_str(&self) -> &'static str {
        match self {
            State::StealStickerSetName => "steal_sticker_set_name",
            State::NewStickerSetName => "new_sticker_set_name",
            State::CreateNewStickerSet => "create_new_sticker_set",
            State::NewStickerSetTitle => "new_sticker_set_title",
        }
    }
}

impl From<State> for Cow<'static, str> {
    fn from(state: State) -> Self {
        Cow::Borrowed(state.as_str())
    }
}

impl PartialEq<&str> for State {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
