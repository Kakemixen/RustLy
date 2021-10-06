use engine::{error, warning, info, debug};
use engine;

fn main() 
{
    error!("hello {}", 2);
    warning!("hello {}", 2);
    info!("hello {}", 2);
    debug!("hello {}", 2);
    engine::test_log();
}
