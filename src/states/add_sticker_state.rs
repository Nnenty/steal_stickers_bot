use std::borrow::Cow;

#[derive(Clone)]
pub enum AddStickerState {
    StealSticker,
    AddStickerToUserOwnedStickerSet,
}

impl AddStickerState {
    const fn as_str(&self) -> &'static str {
        match self {
            AddStickerState::StealSticker => "steal_sticker",
            AddStickerState::AddStickerToUserOwnedStickerSet => {
                "add_sticker_to_user_owned_sticker_set"
            }
        }
    }
}

impl From<AddStickerState> for Cow<'static, str> {
    fn from(state: AddStickerState) -> Self {
        Cow::Borrowed(state.as_str())
    }
}

impl PartialEq<&str> for AddStickerState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
