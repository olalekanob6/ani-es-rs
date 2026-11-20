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
pub use history::{
    latest_anime, load_history, migrate_legacy_db_if_exists, save_history,
    update_history_for_provider,
};
pub use logger::{error, info, init, warn, Level};
pub use models::{HistoryMap, WatchlistEntry, WatchlistMap, WatchlistStatus};
pub use player::{play_video, play_video_with_track_menu};
pub use providers::{AnimeAv1Provider, AnimeProvider, JkanimeProvider};
pub use scraper::{
    build_client, fetch_anime_detail, fetch_anime_detail_with_provider, fetch_video_source,
    fetch_video_source_with_provider, search_anime, search_anime_cached_with_provider,
    search_anime_with_provider,
};
pub use ui::{prompt_search, select_episode, select_from_list, select_post_episode_action};
pub use watchlist::{
    add_entry_for_provider, add_watchlist, add_watchlist_for_provider, ensure_entry_provider,
    find_entry, load_watchlist, mark_status, remove_watchlist, save_watchlist,
};
