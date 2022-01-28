pub use colored::Colorize;
use crossbeam::channel;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use thread_local::ThreadLocal;

pub mod prelude
{
	pub use super::{debug, error, info, log_init, trace, warning};
}

pub mod core_prelude
{
	pub use super::{core_debug, core_error, core_info, core_trace, core_warning};
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

	println!(
		"{}",
		format!(
			"[{:7}{}] {}:{} - {}",
			levelstr,
			corestr,
			event.file,
			event.line,
			event.message.replace("\n", "\n[   -   ] ")
		)
	);
}

type LogSender = channel::Sender<LogEnum>;
fn init_channel() -> LogSender
{
	let (tx, rx) = channel::bounded(20);

	thread::Builder::new()
		.name("LogThread".to_string())
		.spawn(move || {
			for line in rx {
				match line {
					LogEnum::Msg(event) => {
						print_log_event(event);
					}
					LogEnum::Kill(msg) => {
						println!("{}: {}", "Logging abort reason".red(), msg);
						std::process::abort();
					}
				};
			}
		})
		.unwrap();
	tx
}

pub fn log_init()
{
	static INITIALIZED: AtomicBool = AtomicBool::new(false);
	if INITIALIZED.load(Ordering::Relaxed) {
		panic!("Cannot initialize log multiple times");
	}
	INITIALIZED.store(true, Ordering::Relaxed);

	let tx = init_channel();
	let logger_box = Box::new(Logger::new(tx));

	unsafe {
		LOGGER = Box::leak(logger_box);
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
	};

	unsafe {
		LOGGER.log(event);
	}
}

trait Log: Sync
{
	fn log(&self, event: LogEvent);
}

struct EmptyLogger;

impl Log for EmptyLogger
{
	fn log(&self, _event: LogEvent) {}
}

static mut LOGGER: &dyn Log = &EmptyLogger;

struct Logger
{
	transmitter: ThreadLocal<LogSender>,
	tx_main: LogSender,
}

impl Logger
{
	fn new(tx: LogSender) -> Self
	{
		let logger = Logger {
			transmitter: ThreadLocal::new(),
			tx_main: tx,
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
				channel::TrySendError::Full(_) => {
					tx.send(LogEnum::Kill("Full channel".to_string())).unwrap();
				}
				channel::TrySendError::Disconnected(_) => {
					panic!("Disconnected logger, can't send logs");
				}
			};
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
