use rustly::log::*;

fn main()
{
    error!("hello {}", 2);
    warning!("hello {}", 2);
    info!("hello {}", 2);
    debug!("hello {}", 2);
    test_log();
}
