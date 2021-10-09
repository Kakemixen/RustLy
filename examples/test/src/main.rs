use rustly::log;

fn main()
{
    log::error!("hello {}", 2);
    log::warning!("hello {}", 2);
    log::info!("hello {}", 2);
    log::debug!("hello {}", 2);
    log::test_log();
}
