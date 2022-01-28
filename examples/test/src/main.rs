use rustly;
use rustly::log::*;
use std::thread;

fn main()
{
	log_init();

	let handle1 = thread::spawn(|| {
		error!("hello {}", 2);
		warning!("hello {}", 2);
		info!("hello {}", 2);
		debug!("hello {}", 2);
		trace!("hello {}", 2);
	});

	let handle2 = thread::spawn(|| {
		error!("hello \n{}", 2);
		warning!("hello \n{}", 2);
		info!("hello \n{}", 2);
		debug!("hello \n{}", 2);
		trace!("hello \n{}", 2);
	});

	let handle3 = thread::spawn(|| {
		error!("hello \n{}", 2);
		warning!("hello \n{}", 2);
		info!("hello \n{}", 2);
		debug!("hello \n{}", 2);
		trace!("hello \n{}", 2);
	});

	rustly::test_log();

	handle1.join().unwrap();
	handle2.join().unwrap();
	handle3.join().unwrap();
}
