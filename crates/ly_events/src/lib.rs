// implementation from Ross
// https://betterprogramming.pub/rust-events-revisited-926486721e3f
//
// There is a major problem with this implementation
// I cannot pass a closure as the callback, a result of the fn(type) (function
// pointer). This makes it difficult to affect state outside of the callback
// function. I would like to have events that let me update some state, which is
// difficult now

trait Receiver
{
	type Data;
	type Transformer;

	fn on_emit(self, data: Self::Data);
}

trait Signal
{
	type Data;
	type RecType: Receiver;

	fn emit(&self, data: Self::Data);
	fn connect(&mut self, rec: <Self::RecType as Receiver>::Transformer) -> Self::RecType;
	fn disconnect(&mut self, i: usize);
}

#[cfg(test)]
mod tests
{
	use super::*;
	use std::cell::RefCell;
	use std::rc::Rc;

	#[derive(Clone)]
	struct TestEventData
	{
		total: Rc<RefCell<i32>>,
		num: i32,
	}

	#[derive(Clone, Copy)]
	struct TestReciever
	{
		id: usize,
		cls: fn(Self, TestEventData),
	}

	impl TestReciever
	{
		fn new(id: usize, cls: fn(TestReciever, TestEventData)) -> Self { Self { id, cls } }
	}

	impl Receiver for TestReciever
	{
		type Data = TestEventData;
		type Transformer = fn(Self, TestEventData);

		fn on_emit(self, data: Self::Data) { (self.cls)(self, data); }
	}

	struct TestSignal
	{
		next_id: usize,
		recs: Vec<TestReciever>,
	}

	impl TestSignal
	{
		fn nxt(&mut self) -> usize
		{
			self.next_id += 1;
			self.next_id
		}

		fn new() -> Self
		{
			TestSignal {
				recs: Vec::new(),
				next_id: 0,
			}
		}
	}

	impl Signal for TestSignal
	{
		type Data = TestEventData;
		type RecType = TestReciever;

		fn emit(&self, data: Self::Data) { self.recs.iter().for_each(|r| r.on_emit(data.clone())) }
		fn connect(&mut self, rec: <Self::RecType as Receiver>::Transformer) -> Self::RecType
		{
			let i = self.nxt();
			let r = Self::RecType::new(i, rec);
			self.recs.push(r);
			r
		}
		fn disconnect(&mut self, i: usize)
		{
			let idx = self.recs.iter().position(|r| r.id == i).unwrap();
			self.recs.remove(idx);
		}
	}

	#[test]
	fn flow()
	{
		let total = Rc::new(RefCell::new(0));
		let result = 2 + 2;
		assert_eq!(result, 4);
		let mut sig = TestSignal::new();
		let _rec = sig.connect(|_t, x: TestEventData| {
			let t = &*x.total;
			*t.borrow_mut() += x.num;
		});
		sig.emit(TestEventData {
			total: Rc::clone(&total),
			num: 3,
		});
		let tot = *total.borrow();
		assert_eq!(tot, 3)
	}
}
