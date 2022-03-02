//! The LY engine aims to provide an ECS game engine with a vulkan renderer
//!
//! ### This Crate
//! The main crate re-exports the relevant API from the sub-crates,
//! making is more ergonomic to consume the public API.
//!
//! You can use those crates directly if you want,
//! they export some more functionality intended for the engine core.

/// Logging module for LY engine clients
///
/// It contains macros to log format strings via a logging thread
///
/// The logger must be initiaized with the [log::log_init] function
///
/// There are five logging levels/macros, listed in increasing severity:
/// `trace!`, `debug!`, `info!`, `warning!`, `error!`.
///
/// Which log level is used is decided at compile time with the following
/// features, with each feature also disabling all logs of a lower severity:
/// - `strip_trace`
/// - `strip_debug`
/// - `strip_info`
/// - `strip_warning`
///
/// crate doc: [ly_log]
pub mod log
{
	pub use ly_log::prelude::*;
}

use ly_log::core_prelude::*;

/// Application for the engine, the glue that holds everything together.
/// Don't create more that one, it uses references to static variables.
pub mod app
{
	pub use ly_app::*;
}

/// Event system for LY engine clients
///
/// The crate provides functionality to send event via "channels"
/// and event signals to synchonize threads.
///
/// The most important module is [events::channel], which is probably why you
/// are here.
///
/// crate doc: [ly_events]
pub mod events
{
	pub use ly_events::*;
}

/// Input types for LY engine clients
///
/// crate doc: [ly_input]
pub mod input
{
	pub use ly_input::*;
}

/// Window abstraction for LY engine clients
///
/// crate doc: [ly_window]
pub mod window
{
	pub use ly_window::*;
}

pub fn test_log()
{
	core_error!("test {}", 2);
	core_warning!("test {}", 2);
	core_info!("test {}", 2);
	core_debug!("test {}", 2);
	core_trace!("test {}", 2);
}
