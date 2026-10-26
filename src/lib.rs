pub mod colors;
pub mod config;
pub mod history;
pub mod logger;
pub mod models;
pub mod player;
pub mod scraper;
pub mod ui;

pub use colors::{color, BOLD, CYAN, GREEN, MAGENTA, RED, RESET, YELLOW};
pub use config::{Config, AppConfig};
pub use history::{load_history, migrate_legacy_db_if_exists, save_history};
pub use logger::{error, info, init, warn, Level};
pub use models::HistoryMap;
pub use player::play_video;
pub use scraper::{build_client, fetch_anime_detail, fetch_video_source, search_anime};
pub use ui::{prompt_search, select_episode, select_from_list, select_post_episode_action};
