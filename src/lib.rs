pub mod cache;
pub mod colors;
pub mod config;
pub mod history;
pub mod logger;
pub mod models;
pub mod player;
pub mod providers;
pub mod scraper;
pub mod storage;
pub mod ui;
pub mod update;
pub mod watchlist;

pub use cache::ScraperCache;
pub use colors::{color, set_color_enabled, BOLD, CYAN, GREEN, MAGENTA, RED, RESET, YELLOW};
pub use config::{AppConfig, Config};
pub use history::{latest_anime, load_history, migrate_legacy_db_if_exists, save_history};
pub use logger::{error, info, init, warn, Level};
pub use models::{HistoryMap, WatchlistEntry, WatchlistMap, WatchlistStatus};
pub use player::play_video;
pub use scraper::{build_client, fetch_anime_detail, fetch_video_source, search_anime};
pub use ui::{prompt_search, select_episode, select_from_list, select_post_episode_action};
pub use watchlist::{
    add_watchlist, find_entry, load_watchlist, mark_status, remove_watchlist, save_watchlist,
};
