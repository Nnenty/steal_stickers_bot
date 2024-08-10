pub mod cancel;
pub mod start;
pub mod steal_sticker_set;

pub use cancel::cancel_handler;
pub use start::start_handler;
pub use steal_sticker_set::{
    create_new_sticker_set, process_wrong_sticker, steal_handler, steal_sticker_set_handler,
};

// DELETE `// dev` IN FUNCTIONS:
// steal_sticker_set_handler
// get_new_sticker_set_title
// create_new_sticker_set
