pub mod appdata;
pub mod app;
pub mod widgets;
pub mod cmd_queue;
pub mod dgui;
pub mod error;
pub mod ext;
pub mod settings;
pub mod projects;
pub mod util;
pub mod fp;
pub mod project_wizard;
pub mod charcons;
pub mod work_pool;
pub mod macros;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
pub const IS_DEBUG: bool = cfg!(debug_assertions);
pub const HAS_TERMINAL: bool = cfg!(feature = "use_terminal");