use rustly::log::*;
use std::thread;

fn main()
{
    init();

    let handle = thread::spawn(|| {
        rustly::test_log();
    });

    let handle2 = thread::spawn(|| {
        error!("hello {}", 2);
        warning!("hello {}", 2);
        info!("hello {}", 2);
        debug!("hello {}", 2);
        trace!("hello {}", 2);
    });

    let handle3 = thread::spawn(|| {
        error!("hello {}", 2);
        warning!("hello {}", 2);
        info!("hello {}", 2);
        debug!("hello {}", 2);
        trace!("hello {}", 2);
    });

    handle.join().unwrap();
    handle2.join().unwrap();
    handle3.join().unwrap();
}
