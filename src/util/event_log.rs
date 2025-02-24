use std::time::Duration;

pub const RESET: &str = "\x1b[0m";
pub const BLUE: &str = "\x1b[34m";
pub const GREEN: &str = "\x1b[32m";
pub const CYAN: &str = "\x1b[36m";
pub const YELLOW: &str = "\x1b[33m";
pub const MAGENTA: &str = "\x1b[35m";
pub const RED: &str = "\x1b[31m";

#[derive(Debug)]
pub enum Event {
    Info,
    TotalElapsed,
    ParseChunks,
    CollectImageChunks,
    FlateDecompress,
    RowFilters,
}

impl Event {
    const fn color(&self) -> &'static str {
        match self {
            Self::Info => "",
            Self::TotalElapsed => YELLOW,
            Self::ParseChunks => MAGENTA,
            Self::CollectImageChunks => CYAN,
            Self::FlateDecompress => GREEN,
            Self::RowFilters => BLUE,
        }
    }
}

pub fn log_event(msg: &str, event: Event, duration: Option<Duration>) {
    if let Some(duration) = duration {
        println!("{}{:?}\t{:?}\t{}", event.color(), duration, event, msg);
    } else {
        println!("{}{:?}\t{}", event.color(), event, msg)
    }

    if matches!(event, Event::TotalElapsed) {
        println!("{}", RESET);
    }
}
