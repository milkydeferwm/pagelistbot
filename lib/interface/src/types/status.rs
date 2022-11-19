//! Status types.

pub mod finder;
pub mod refresher;
pub mod task;

// Re-exports
pub use finder::PageListBotTaskFinderStatus;
pub use refresher::PageListBotRefresherStatus;
pub use task::PageListBotTaskStatus;
