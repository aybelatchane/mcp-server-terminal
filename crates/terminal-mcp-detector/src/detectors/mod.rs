//! Element detector implementations.

pub mod border;
pub mod button;
pub mod checkbox;
pub mod input;
pub mod menu;
pub mod progress;
pub mod status_bar;
pub mod table;

pub use border::BorderDetector;
pub use button::ButtonDetector;
pub use checkbox::CheckboxDetector;
pub use input::InputDetector;
pub use menu::MenuDetector;
pub use progress::ProgressDetector;
pub use status_bar::StatusBarDetector;
pub use table::TableDetector;
