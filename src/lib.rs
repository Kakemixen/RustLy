pub mod log
{
    pub use ly_log::prelude::*;
}

use ly_log::core_prelude::*;

pub fn test_log()
{
    core_error!("test {}", 2);
    core_warning!("test {}", 2);
    core_info!("test {}", 2);
    core_debug!("test {}", 2);
    core_trace!("test {}", 2);
}

