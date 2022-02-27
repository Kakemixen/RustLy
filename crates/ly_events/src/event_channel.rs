use std::cell::UnsafeCell;

#[derive(Debug)]
pub(crate) enum ReadableEventBuffer
{
	A,
	B,
}

/// Single-threaded event channel
pub struct EventChannel<T>
{
	pub(crate) events_a: UnsafeCell<Vec<T>>,
	pub(crate) events_b: UnsafeCell<Vec<T>>,
	pub(crate) start_idx_a: UnsafeCell<usize>,
	pub(crate) start_idx_b: UnsafeCell<usize>,
	pub(crate) readable_buffer: UnsafeCell<ReadableEventBuffer>,
	writers: UnsafeCell<usize>,
}

/// Single-threaded event writer
///
/// Created by [`EventChannel::get_writer`].
/// Borrows the channel immutably upon creation.
pub struct EventWriter<'a, T>
{
	channel: &'a EventChannel<T>,
}

/// Single-threaded event reader
///
/// Created by [`EventChannel::get_reader`].
/// Borrows the channel immutably upon creation.
pub struct EventReader<'a, T>
{
	read_events: UnsafeCell<usize>,
	channel: &'a EventChannel<T>,
}

impl<T> EventChannel<T>
{
	/// Creates empty event channel
	pub fn new() -> EventChannel<T>
	{
		EventChannel {
			events_a: UnsafeCell::new(Vec::new()), // maybe sensible initial?
			events_b: UnsafeCell::new(Vec::new()), // maybe sensible initial?
			start_idx_a: UnsafeCell::new(0),
			start_idx_b: UnsafeCell::new(0),
			readable_buffer: UnsafeCell::new(ReadableEventBuffer::A),
			writers: UnsafeCell::new(0),
		}
	}

	/// Sends the event on the channel
	pub(crate) fn send(&self, e: T)
	{
		unsafe {
			match *self.readable_buffer.get() {
				ReadableEventBuffer::A => {
					(*self.events_b.get()).push(e);
					(*self.start_idx_b.get()) += 1;
				}
				ReadableEventBuffer::B => {
					(*self.events_a.get()).push(e);
					(*self.start_idx_a.get()) += 1;
				}
			}
		}
	}

	/// Flushes events on the channel
	///
	/// Makes the currently sent un-flushed events readable.
	///
	/// This drops all previously flushed events, making them unreadable.
	///
	/// Is is adviced to let one of the readers initiate the flush with
	/// [`EventReader::flush_channel`],
	/// as they are controlling consumation of events.
	pub fn flush(&self)
	{
		let readable_buffer = self.readable_buffer.get();
		unsafe {
			match *readable_buffer {
				ReadableEventBuffer::A => {
					(*self.events_a.get()).clear();
					*readable_buffer = ReadableEventBuffer::B;

					*self.start_idx_a.get() = *self.start_idx_b.get() // so that reading starts counting properly
				}
				ReadableEventBuffer::B => {
					(*self.events_b.get()).clear();
					*readable_buffer = ReadableEventBuffer::A;

					*self.start_idx_b.get() = *self.start_idx_a.get()
				}
			}
		}
	}

	/// Creates a writer for this channel
	pub fn get_writer(&self) -> EventWriter<T>
	{
		unsafe {
			let writers = self.writers.get();
			*writers += 1;
		}
		EventWriter { channel: self }
	}

	/// Creates a reader for this channel
	pub fn get_reader(&self) -> EventReader<T>
	{
		EventReader {
			read_events: UnsafeCell::new(0),
			channel: self,
		}
	}

	fn has_writers(&self) -> bool
	{
		unsafe {
			let writers = self.writers.get();
			*writers != 0
		}
	}
}

impl<'a, T> EventWriter<'a, T>
{
	/// Sends the event to the channel
	pub fn send(&self, event: T) { self.channel.send(event); }
}

impl<'a, T> Drop for EventWriter<'a, T>
{
	fn drop(&mut self)
	{
		unsafe {
			let writers = self.channel.writers.get();
			*writers += 1;
		}
	}
}

impl<'a, T> EventReader<'a, T>
{
	/// Reads all unread events from this channel
	///
	/// Giver an `Iterator` over the currently flushed events.
	///
	/// Becaus of how this is setup, it reads all flushed events, or none at all
	/// if the flushed events have been read by this reader.
	pub fn read(&self) -> impl Iterator<Item = &T>
	{
		unsafe {
			let readable_buffer = self.channel.readable_buffer.get();
			let read_events = self.read_events.get();
			match *readable_buffer {
				ReadableEventBuffer::A => {
					let start_idx_a = *self.channel.start_idx_a.get();
					if *read_events > start_idx_a {
						[].iter()
					}
					else {
						*read_events = start_idx_a + 1;
						(*self.channel.events_a.get()).iter()
					}
				}
				ReadableEventBuffer::B => {
					let start_idx_b = *self.channel.start_idx_b.get();
					if *read_events > start_idx_b {
						[].iter()
					}
					else {
						*read_events = start_idx_b + 1;
						(*self.channel.events_b.get()).iter()
					}
				}
			}
		}
	}

	/// Initiates a flush on the reader's connected channel
	///
	/// It is adviced to use this for flushing. Read [`EventChannel::flush`]
	/// for a descripion of the behaviour regarding flushing.
	pub fn flush_channel(&self) { self.channel.flush(); }

	/// Checks if there are any writers connected to reading channel
	pub fn channel_has_writers(&self) -> bool { self.channel.has_writers() }
}
