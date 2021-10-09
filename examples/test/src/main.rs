use rustly::ly_log;

fn main()
{
    ly_log::error!("hello {}", 2);
    ly_log::warning!("hello {}", 2);
    ly_log::info!("hello {}", 2);
    ly_log::debug!("hello {}", 2);
    ly_log::test_log();
}
