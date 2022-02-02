use rustly::ly_events;
use std::time::Instant;

struct MyEvent
{
	num: usize,
}

const ITERATIONS: usize = 10;
const READ_BATCH: usize = 20;
const NUM_EVENTS: usize = 1000000000;

fn run() -> usize
{
	let channel = ly_events::EventChannel::<MyEvent>::new();
	let reader = channel.get_reader();
	let mut total: usize = 0;

	let time = Instant::now();
	for i in 0..NUM_EVENTS {
		let event = MyEvent { num: i };
		channel.send(event);

		if i % READ_BATCH == 0 {
			channel.flush();
			for e in reader.iter() {
				total += e.num;
			}
		}
	}

	let millis = time.elapsed().as_millis() as usize;
	println!("total: {}", total);
	millis
}

fn main()
{
	let mut times = Vec::with_capacity(ITERATIONS);
	for i in 0..ITERATIONS {
		let time = run();
		times.push(time);
	}
	println!("total time: {}", times.iter().sum::<usize>());
}

// Results
// config:
// const ITERATIONS: usize = 10;
// const READ_BATCH: usize = 20;
// const NUM_EVENTS: usize = 1000000000;
//
// no interior mutability:    34572 ms
// RefCell implementation:    46615 ms
// UnsafeCell implementation: 21074 ms