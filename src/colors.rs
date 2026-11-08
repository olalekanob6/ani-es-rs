pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";
pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const MAGENTA: &str = "\x1b[35m";
pub const CYAN: &str = "\x1b[36m";

pub fn color<T: AsRef<str>>(text: T, code: &str) -> String {
    if COLOR_ENABLED.load(Ordering::Relaxed)
        && atty::is(atty::Stream::Stdout)
        && std::env::var_os("NO_COLOR").is_none()
    {
        format!("{}{}{}", code, text.as_ref(), RESET)
    } else {
        text.as_ref().to_string()
    }
}

pub fn set_color_enabled(enabled: bool) {
    COLOR_ENABLED.store(enabled, Ordering::Relaxed);
}
use std::sync::atomic::{AtomicBool, Ordering};

static COLOR_ENABLED: AtomicBool = AtomicBool::new(false);
