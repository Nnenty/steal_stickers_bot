use std::borrow::Cow;

#[derive(Clone)]
pub enum State {
    Sticker,
}

impl State {
    const fn as_str(&self) -> &'static str {
        match self {
            State::Sticker => "sticker",
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
