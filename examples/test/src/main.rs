use rustly::log::*;
use std::thread;

fn main()
{
    init();

    let handle1 = thread::spawn(|| {
        error!("hello {}", 2);
        thread::sleep_ms(2);
        warning!("hello {}", 2);
        thread::sleep_ms(2);
        info!("hello {}", 2);
        thread::sleep_ms(2);
        debug!("hello {}", 2);
        thread::sleep_ms(2);
        trace!("hello {}", 2);
        thread::sleep_ms(2);
    });

    let handle2 = thread::spawn(|| {
        error!("hello \n{}", 2);
        thread::sleep_ms(2);
        warning!("hello \n{}", 2);
        thread::sleep_ms(2);
        info!("hello \n{}", 2);
        thread::sleep_ms(2);
        debug!("hello \n{}", 2);
        thread::sleep_ms(2);
        trace!("hello \n{}", 2);
        thread::sleep_ms(2);
    });

    handle1.join().unwrap();
    handle2.join().unwrap();
}
