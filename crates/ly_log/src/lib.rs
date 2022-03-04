//! Logging system in the LY engine
//!
//! It contains macros to log format strings via a logging thread
//!
//! The logger must be initiaized with the [`log_init`] function
//!
//! There are five logging levels/macros, listed in increasing severity:
//! `trace!`, `debug!`, `info!`, `warning!`, `error!`.
//!
//! Which log level is used is decided at compile time with the following
//! features, with each feature also disabling all logs of a lower severity:
//! - strip_trace
//! - strip_debug
//! - strip_info
//! - strip_warning
//!
//! Logs will indicate if they blocked the sender side.
//! Can be dissallowed with the feature `dissallow_blocking`,
//! in which case blocking events will panic.
//!
//! ### Engine API
//!
//! The engine should use the [core_prelude], which will export
//! macros with the `core_` prefix, and indicate that the log
//! originated from the engine.
//!
//! The logger should be cleaned up with the [`log_die`] function,
//! but it is not strictly necessary. It sends a kill command,
//! and waits for the logger to finish all currently received logs.
//! If the main threads exit without calling this, logs will be lost.
//! Make sure all threads generating logs are stopped before calling
//! this method.

pub use colored::Colorize;
use crossbeam::channel;
use parking_lot::{Condvar, Mutex};
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use thread_local::ThreadLocal;

/// exports intended for clients outside the LY engine
pub mod prelude
{
	pub use super::{debug, error, info, trace, warning};
}

/// exports intended for the LY engine
pub mod core_prelude
{
	pub use super::{
		core_debug, core_error, core_info, core_trace, core_warning, log_die, log_init,
	};
}

pub enum LogLevel
{
	Error,
	Warning,
	Info,
	Debug,
	Trace,
}

struct LogEvent
{
	level: LogLevel,
	in_core: bool,
	blocking: bool,
	file: &'static str,
	line: u32,
	message: String,
}

enum LogEnum
{
	Msg(LogEvent),
	Kill(String),
}

fn print_log_event(event: LogEvent)
{
	let levelstr = match event.level {
		LogLevel::Error => "ERROR".red(),
		LogLevel::Warning => "WARNING".yellow(),
		LogLevel::Info => "INFO".green(),
		LogLevel::Debug => "DEBUG".blue(),
		LogLevel::Trace => "TRACE".truecolor(80, 80, 80),
	};
	let corestr = match event.in_core {
		true => " LY".magenta(),
		false => "".normal(),
	};
	let blockingstr = match event.blocking {
		true => " B!".red(),
		false => "".normal(),
	};

	println!(
		"{}",
		format!(
			"[{:7}{}{}] {}:{} - {}",
			levelstr,
			corestr,
			blockingstr,
			event.file,
			event.line,
			event
				.message
				.replace("\n", &format!("\n[   -   {}{}] ", corestr, blockingstr))
		)
	);
}

type CondPair = Arc<(Mutex<bool>, Condvar)>;

fn print_log_die(msg: String, condpair: CondPair)
{
	let levelstr = "INFO".green();
	let corestr = " LY".magenta();

	println!(
		"{}",
		format!(
			"[{:7}{}] Stopping log thread - {}",
			levelstr,
			corestr,
			msg.replace("\n", &format!("\n[   -   {}] ", corestr))
		)
	);
	let (lock, cvar) = &*condpair;
	let mut finished = lock.lock();
	*finished = true;
	// We notify the condvar that the value has changed.
	cvar.notify_one();
}

type LogSender = channel::Sender<LogEnum>;
fn init_channel() -> (LogSender, CondPair)
{
	let (tx, rx) = channel::bounded(6);
	let pair = Arc::new((Mutex::new(false), Condvar::new()));
	let pair2 = Arc::clone(&pair);

	thread::Builder::new()
		.name("LogThread".to_string())
		.spawn(move || {
			for line in rx {
				match line {
					LogEnum::Msg(event) => {
						print_log_event(event);
					}
					LogEnum::Kill(msg) => {
						print_log_die(msg, pair2);
						break;
					}
				};
			}
		})
		.unwrap();
	(tx, pair)
}

// TODO make only public in engine once Application is up and running
/// initializes the global logger with it's own logging thread
pub fn log_init()
{
	static INITIALIZED: AtomicBool = AtomicBool::new(false);
	if INITIALIZED.load(Ordering::Relaxed) {
		panic!("Cannot initialize log multiple times");
	}
	INITIALIZED.store(true, Ordering::Relaxed);

	let logger_box = Box::new(Logger::new());

	unsafe {
		LOGGER = Box::leak(logger_box);
	}
}

/// Stops the logger and cleans up
///
/// This will make all following calls to logging panic
/// make sure you have cleaned up all threads when this is called
pub fn log_die(message: String)
{
	unsafe {
		LOGGER.log_die(message);
	}
}

#[doc(hidden)]
pub fn __private_log(
	in_core: bool,
	level: LogLevel,
	file: &'static str,
	line: u32,
	args: fmt::Arguments,
)
{
	let event = LogEvent {
		level,
		in_core,
		file,
		line,
		message: format!("{}", args),
		blocking: false,
	};

	unsafe {
		LOGGER.log(event);
	}
}

trait Log: Sync
{
	fn log(&self, event: LogEvent);
	fn log_die(&self, message: String);
}

struct EmptyLogger;

impl Log for EmptyLogger
{
	fn log(&self, _event: LogEvent) {}
	fn log_die(&self, _msg: String) {}
}

static mut LOGGER: &dyn Log = &EmptyLogger;

struct Logger
{
	transmitter: ThreadLocal<LogSender>,
	tx_main: LogSender,
	condpair: CondPair,
}

impl Logger
{
	fn new() -> Self
	{
		let (tx, condpair) = init_channel();
		let logger = Logger {
			transmitter: ThreadLocal::new(),
			tx_main: tx,
			condpair,
		};
		logger
	}
}

impl Log for Logger
{
	fn log(&self, event: LogEvent)
	{
		let tx = self.transmitter.get_or(|| self.tx_main.clone());
		if let Err(e) = tx.try_send(LogEnum::Msg(event)) {
			match e {
				#[cfg(not(feature = "disallow_blocking"))]
				channel::TrySendError::Full(LogEnum::Msg(e)) => {
					let blocking_event = LogEvent {
						blocking: tx.is_full(),
						..e
					};
					tx.send(LogEnum::Msg(blocking_event)).unwrap();
				}
				#[cfg(not(feature = "disallow_blocking"))]
				channel::TrySendError::Full(LogEnum::Kill(_)) => {
					panic!("Could not send log kill event, this really should not happen!");
				}
				#[cfg(feature = "disallow_blocking")]
				channel::TrySendError::Full(LogEnum::Msg(e)) => {
					tx.send(LogEnum::Kill("Full channel".to_string())).unwrap();
					self.log_die("Disallowed blocking, killed");
					panic!();
				}
				#[cfg(feature = "disallow_blocking")]
				channel::TrySendError::Full(LogEnum::Kill(e)) => {
					tx.send(LogEnum::Kill(e)).unwrap();
					self.log_die("Disallowed blocking, killed resend");
					panic!();
				}
				channel::TrySendError::Disconnected(_) => {
					panic!("Disconnected logger, can't send logs");
				}
			};
		}
	}

	fn log_die(&self, msg: String)
	{
		let tx = self.transmitter.get_or(|| self.tx_main.clone());
		if let Err(channel::SendError(_)) = tx.send(LogEnum::Kill(msg)) {
			panic!("Disconnected logger, could not send log_die event, this should not happen!");
		}
		let (lock, cvar) = &*self.condpair;
		let mut finished = lock.lock();
		// We notify the condvar that the value has changed.
		while !*finished {
			cvar.wait(&mut finished);
		}
	}
}

// macros

#[macro_export]
macro_rules! error
{
    () => { };
    ($($x : tt) *) => { $crate::__private_log(
            false,
            $crate::LogLevel::Error,
            file!(), line!(),
            format_args!(
                $($x) *
                )
            ) };
}

#[cfg(not(feature = "strip_warning"))]
#[macro_export]
macro_rules! warning
{
    () => { };
    ($($x : tt) *) => { $crate::__private_log(
            false,
            $crate::LogLevel::Warning,
            file!(), line!(),
            format_args!(
                $($x) *
                )
            ) };
}

#[cfg(feature = "strip_warning")]
#[macro_export]
macro_rules! warning {
	($($x:tt)*) => {};
}

#[cfg(not(feature = "strip_info"))]
#[macro_export]
macro_rules! info
{
    () => { };
    ($($x : tt) *) => { $crate::__private_log(
            false,
            $crate::LogLevel::Info,
            file!(), line!(),
            format_args!(
                $($x) *
                )
            ) };
}

#[cfg(feature = "strip_info")]
#[macro_export]
macro_rules! info {
	($($x:tt)*) => {};
}

#[cfg(not(feature = "strip_debug"))]
#[macro_export]
macro_rules! debug
{
    () => { };
    ($($x : tt) *) => { $crate::__private_log(
            false,
            $crate::LogLevel::Debug,
            file!(), line!(),
            format_args!(
                $($x) *
                )
            ) };
}

#[cfg(feature = "strip_debug")]
#[macro_export]
macro_rules! debug {
	($($x:tt)*) => {};
}

#[cfg(not(feature = "strip_trace"))]
#[macro_export]
macro_rules! trace
{
    () => { };
    ($($x : tt) *) => { $crate::__private_log(
            false,
            $crate::LogLevel::Trace,
            file!(), line!(),
            format_args!(
                $($x) *
                )
            ) };
}

#[cfg(feature = "strip_trace")]
#[macro_export]
macro_rules! trace {
	($($x:tt)*) => {};
}

#[macro_export]
macro_rules! core_error
{
    () => { };
    ($($x : tt) *) => { $crate::__private_log(
            true,
            $crate::LogLevel::Error,
            file!(), line!(),
            format_args!(
                $($x) *
                )
            ) };
}

#[cfg(not(feature = "strip_warning"))]
#[macro_export]
macro_rules! core_warning
{
    () => { };
    ($($x : tt) *) => { $crate::__private_log(
            true,
            $crate::LogLevel::Warning,
            file!(), line!(),
            format_args!(
                $($x) *
                )
            ) };
}

#[cfg(feature = "strip_warning")]
#[macro_export]
macro_rules! core_warning {
	($($x:tt)*) => {};
}

#[cfg(not(feature = "strip_info"))]
#[macro_export]
macro_rules! core_info
{
    () => { };
    ($($x : tt) *) => { $crate::__private_log(
            true,
            $crate::LogLevel::Info,
            file!(), line!(),
            format_args!(
                $($x) *
                )
            ) };
}

#[cfg(feature = "strip_info")]
#[macro_export]
macro_rules! core_info {
	($($x:tt)*) => {};
}

#[cfg(not(feature = "strip_debug"))]
#[macro_export]
macro_rules! core_debug
{
    () => { };
    ($($x : tt) *) => { $crate::__private_log(
            true,
            $crate::LogLevel::Debug,
            file!(), line!(),
            format_args!(
                $($x) *
                )
            ) };
}

#[cfg(feature = "strip_debug")]
#[macro_export]
macro_rules! core_debug {
	($($x:tt)*) => {};
}

#[cfg(not(feature = "strip_trace"))]
#[macro_export]
macro_rules! core_trace
{
    () => { };
    ($($x : tt) *) => { $crate::__private_log(
            true,
            $crate::LogLevel::Trace,
            file!(), line!(),
            format_args!(
                $($x) *
                )
            ) };
}

#[cfg(feature = "strip_trace")]
#[macro_export]
macro_rules! core_trace {
	($($x:tt)*) => {};
}
