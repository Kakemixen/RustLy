use crossbeam::sync::{Parker, Unparker};
use ly_log::core_prelude::*;
use parking_lot::{Mutex, RwLock, RwLockReadGuard};
use std::cell::UnsafeCell;
use std::slice::Iter;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use crate::channel::{EventChannel, ReadableEventBuffer};
use crate::event_signal;

/// Thread-safe event channel
///
/// The event channel is provided as [`Sync`], but it still placed on the
/// stack unless wrapped in `Arc` or something similar. This is important
/// information regarding having the reader and emitting channel in different
/// threads, keep in mind the reader has a borrow to the channel.
pub struct SyncEventChannel<T>
{
	channel_id: usize,
	channel: EventChannel<T>,
	write_mutex: Mutex<()>,
	flush_mutex: RwLock<()>,
	new_event_waiters: UnsafeCell<Vec<Unparker>>,
	flushed_waiters: UnsafeCell<Vec<Unparker>>,
	writers: UnsafeCell<AtomicUsize>,
}

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
impl<T> Default for SyncEventChannel<T>
{
	fn default() -> Self
	{
		let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
		core_debug!("constructing channel {}", id);
		SyncEventChannel {
			channel_id: id,
			channel: EventChannel::default(),
			write_mutex: Mutex::new(()),
			flush_mutex: RwLock::new(()),
			new_event_waiters: UnsafeCell::new(Vec::new()),
			flushed_waiters: UnsafeCell::new(Vec::new()),
			writers: UnsafeCell::new(AtomicUsize::new(0)),
		}
	}
}

/// Thread-safe event writer
///
/// Created by [`SyncEventChannel::get_writer`].
/// Borrows the channel immutably upon creation.
pub struct SyncEventWriter<'a, T>
{
	channel: &'a SyncEventChannel<T>,
}

/// Thread-safe event reader
///
/// Created by [`SyncEventChannel::get_reader`].
/// Borrows the channel immutably upon creation.
pub struct SyncEventReader<'a, T>
{
	read_events: UnsafeCell<usize>,
	channel: &'a SyncEventChannel<T>,
}

unsafe impl<T> Sync for SyncEventChannel<T> {}

/// Trait for parking the thread and wait for some future event
///
/// Trait is intended for use as trait objects in tandemn with [wait_any_new],
/// but can of course be used for what you like.
pub trait EventWaiter
{
	/// Add the parker to be notified on some future event
	///
	/// Returns an error if not all current events are handled
	fn add_unparker_new(&self, p: &Parker) -> Result<(), String>;
}

impl<'a, T> EventWaiter for SyncEventReader<'a, T>
{
	/// Add the parker to be notified on the next [SyncEventWriter::send].
	///
	/// It is advised to use [wait_any_new] instead, which wraps this function.
	fn add_unparker_new(&self, p: &Parker) -> Result<(), String>
	{
		let _lock = self.channel.write_mutex.lock();
		if self.channel.has_new_events() {
			return Err("already new unflushed events".to_string());
		}

		unsafe {
			event_signal::add_waiter(&mut *self.channel.new_event_waiters.get(), p);
			Ok(())
		}
	}
}

/// Wait for any events to be sent to the channels of the passed
/// [`EventWaiter`]s
///
/// The trait object is used to enable iteration over multiple channel types,
/// perhaps there's a better way, but I don't know about it.
///
/// If any channels has unread events, it will return directly, without waiting
pub fn wait_any_new(readers: &[&dyn EventWaiter])
{
	let p = Parker::new();
	for reader in readers {
		if reader.add_unparker_new(&p).is_err() {
			return;
		}
	}
	p.park();
}

/// Like [`wait_any_new`], but with a timeout in ms
pub fn wait_any_new_timeout(readers: &[&dyn EventWaiter], timeout_ms: u64)
{
	let p = Parker::new();
	for reader in readers {
		if reader.add_unparker_new(&p).is_err() {
			return;
		}
	}
	p.park_timeout(Duration::from_millis(timeout_ms));
}

impl<T> SyncEventChannel<T>
{
	/// Sends the event to the channel
	///
	/// This also wakes any threads waiting for new events via
	/// [`SyncEventReader::wait_new`].
	fn send(&self, e: T)
	{
		let _lock = self.write_mutex.lock();
		self.channel.send(e);
		unsafe {
			let waiters = &mut *self.new_event_waiters.get();
			event_signal::signal_waiters(waiters);
		}
	}

	/// Flushes the channel
	///
	/// Makes the currently sent un-flushed events readable.
	///
	/// This drops all previously flushed events, making them unreadable.
	///
	/// This also wakes any threads waiting for a flush via
	/// [`SyncEventReader::wait_flushed`].
	///
	/// Is is adviced to let one of the readers initiate the flush with
	/// [`SyncEventReader::flush_channel`],
	/// as they are controlling consumation of events.
	pub fn flush(&self)
	{
		let _lock = self.flush_mutex.write();
		self.channel.flush();
		unsafe {
			event_signal::signal_waiters(&mut *self.flushed_waiters.get());
		}
	}

	/// Creates a writer for this channel
	pub fn get_writer(&self) -> SyncEventWriter<T>
	{
		core_debug!("getting writer for channel {}", self.channel_id);
		unsafe {
			let writers = self.writers.get();
			(*writers).fetch_add(1, Ordering::Relaxed);
		}
		SyncEventWriter { channel: self }
	}

	/// Creates a reader for this channel
	pub fn get_reader(&self) -> SyncEventReader<T>
	{
		core_debug!("getting reader for channel {}", self.channel_id);
		SyncEventReader {
			read_events: UnsafeCell::new(1), // avoid stupid stuff when read=0
			channel: self,
		}
	}

	// expects the write_mutex to already be locked by this thread
	// only called from reader.wait_new()
	fn has_new_events(&self) -> bool
	{
		unsafe {
			let buffer = self.channel.readable_buffer.get();
			match *buffer {
				ReadableEventBuffer::A => !(*self.channel.events_b.get()).is_empty(),
				ReadableEventBuffer::B => !(*self.channel.events_a.get()).is_empty(),
			}
		}
	}

	fn has_writers(&self) -> bool
	{
		unsafe {
			let writers = self.writers.get();
			(*writers).load(Ordering::Relaxed) != 0
		}
	}
}

impl<'a, T> SyncEventWriter<'a, T>
{
	/// Sends the event to the channel
	///
	/// This also wakes any threads waiting for new events via
	/// [`SyncEventReader::wait_new`].
	pub fn send(&self, event: T) { self.channel.send(event); }
}

impl<'a, T> Drop for SyncEventWriter<'a, T>
{
	fn drop(&mut self)
	{
		unsafe {
			let writers = self.channel.writers.get();
			(*writers).fetch_sub(1, Ordering::Relaxed);
			if (*writers).load(Ordering::Relaxed) == 0 {
				let _lock = self.channel.write_mutex.lock();
				event_signal::signal_waiters(&mut *self.channel.new_event_waiters.get());
				event_signal::signal_waiters(&mut *self.channel.flushed_waiters.get());
			}
		}
	}
}

impl<'a, T> SyncEventReader<'a, T>
{
	/// Reads all unread events from this channel
	///
	/// Giver an `Iterator` over the currently flushed events.
	///
	/// Becaus of how this is setup, it reads all flushed events, or none at all
	/// if the flushed events have been read by this reader.
	pub fn read(&self) -> impl Iterator<Item = &T>
	{
		let read_lock = self.channel.flush_mutex.read();

		if !self.has_unread() {
			return SyncEventIterator {
				read_lock,
				iterator: [].iter(),
			};
		}

		let channel = &self.channel.channel;
		unsafe {
			let readable_buffer = channel.readable_buffer.get();
			let read_events = self.read_events.get();
			let iterator = match *readable_buffer {
				ReadableEventBuffer::A => {
					let start_idx_a = *channel.start_idx_a.get();
					*read_events = start_idx_a + 1;
					(*channel.events_a.get()).iter()
				}
				ReadableEventBuffer::B => {
					let start_idx_b = *channel.start_idx_b.get();
					*read_events = start_idx_b + 1;
					(*channel.events_b.get()).iter()
				}
			};
			SyncEventIterator {
				read_lock,
				iterator,
			}
		}
	}

	// expects write_mutex to already be locked
	fn has_unread(&self) -> bool
	{
		let channel = &self.channel.channel;
		unsafe {
			let readable_buffer = channel.readable_buffer.get();
			let read_events = self.read_events.get();
			let start_idx = match *readable_buffer {
				ReadableEventBuffer::A => channel.start_idx_a.get(),
				ReadableEventBuffer::B => channel.start_idx_b.get(),
			};
			*read_events <= *start_idx
		}
	}

	/// Initiates a flush on the reader's connected channel
	///
	/// It is adviced to use this for flushing. Read [`EventChannel::flush`]
	/// for a descripion of the behaviour regarding flushing.
	pub fn flush_channel(&self) { self.channel.flush(); }

	/// Waits for un-flushed events to be present
	///
	/// If there already are un-flushed events, this returns directly,
	/// as there are new events that can be flushed.
	///
	/// If no events are present, the thread will halt and wake when the
	/// next [`SyncEventWriter::send`] occurs.
	pub fn wait_new(&self)
	{
		let _lock = self.channel.write_mutex.lock();
		if self.channel.has_new_events() {
			return;
		}

		unsafe {
			let p = Parker::new();
			event_signal::add_waiter(&mut *self.channel.new_event_waiters.get(), &p);
			drop(_lock);
			p.park();
		}
	}

	/// Waits for un-flushed events to be present
	///
	/// Like [`wait_new`](SyncEventReader::wait_new), with a timeout in ms
	pub fn wait_new_timeout(&self, timeout_ms: u64)
	{
		let _lock = self.channel.write_mutex.lock();
		if self.channel.has_new_events() {
			return;
		}

		unsafe {
			let p = Parker::new();
			event_signal::add_waiter(&mut *self.channel.new_event_waiters.get(), &p);
			drop(_lock);
			p.park_timeout(Duration::from_millis(timeout_ms));
		}
	}

	/// Waits for flushed un-read events to be present
	///
	/// This returns directly if the read has not read the currently flushed
	/// events
	///
	/// If the reader has read current events, it will halt and wake when the
	/// next [`SyncEventChannel::flush`] occurs.
	///
	/// Note: This may lead to a deadlock if this thread is responsible for
	/// flushing, but you already knew that.
	pub fn wait_flushed(&self)
	{
		let _lock = self.channel.flush_mutex.write();
		if self.has_unread() {
			return;
		}

		unsafe {
			let p = Parker::new();
			event_signal::add_waiter(&mut *self.channel.flushed_waiters.get(), &p);
			drop(_lock);
			p.park();
		}
	}

	/// Checks if there are any writers connected to reading channel
	pub fn channel_has_writers(&self) -> bool { self.channel.has_writers() }
}

struct SyncEventIterator<'a, T>
{
	#[allow(dead_code)] // keep lock alive while iterating
	read_lock: RwLockReadGuard<'a, ()>,
	iterator: Iter<'a, T>,
}

impl<'a, T> Iterator for SyncEventIterator<'a, T>
{
	type Item = &'a T;

	fn next(&mut self) -> Option<Self::Item> { self.iterator.next() }
}
