#[macro_use]
mod log;

pub fn test_log()
{
    error!("test {}", 2);
    warning!("test {}", 2);
    info!("test {}", 2);
    debug!("test {}", 2);
}
