pub mod log
{
	pub use ly_log::prelude::*;
}

use ly_log::core_prelude::*;

pub mod events
{
	pub use ly_events::*;
}

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
