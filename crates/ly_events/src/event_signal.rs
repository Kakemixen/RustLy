use crossbeam::sync::{Parker, Unparker};
use parking_lot::Mutex;

pub struct SignalEvent
{
	waiters: Mutex<Vec<Unparker>>,
}

impl SignalEvent
{
	pub fn new() -> SignalEvent
	{
		SignalEvent {
			waiters: Mutex::new(Vec::new()),
		}
	}

	pub fn signal(&self)
	{
		println!("locking waiters");
		let mut waiters = self.waiters.lock();
		println!("locked waiters");
		for waiter in waiters.iter() {
			println!("unparking");
			waiter.unpark();
		}
		waiters.clear();
	}

	pub fn wait(&self)
	{
		let p = Parker::new();
		let u = p.unparker().clone();
		{
			self.waiters.lock().push(u);
		}
		println!("parking");
		p.park();
	}
}

#[cfg(test)]
mod tests
{
	use super::*;
	use std::ops::AddAssign;
	use std::sync::Arc;
	use std::thread;
	use std::time::Duration;

	#[test]
	fn t001()
	{
		let total = Arc::new(Mutex::new(0));
		let t = Arc::clone(&total);
		let signal = Arc::new(SignalEvent::new());
		let s = Arc::clone(&signal);

		let adder = thread::spawn(move || {
			for _ in 1..3 {
				s.wait();
				t.lock().add_assign(1);
			}
		});

		{
			let tlock = total.lock();
			assert!(tlock.eq(&0));
		}

		// current implementation doesn't deal with signals before waits
		thread::sleep(Duration::from_millis(10));

		for i in 1..3 {
			signal.signal();
			thread::sleep(Duration::from_millis(10));
			{
				assert!(total.lock().eq(&i));
			}
		}
		adder.join().unwrap();
	}
}
