pub use colored::Colorize;
use std::fmt;
use std::sync::{mpsc,
    atomic::{AtomicBool, Ordering}
};
use std::thread;
use thread_local::ThreadLocal;

pub mod prelude
{
    pub use super::{error, warning, info, debug, trace, init};
}

pub mod core_prelude
{
    pub use super::{core_error, core_warning, core_info, core_debug, core_trace};
}

pub enum LogLevel
{
    Error,
    Warning,
    Info,
    Debug,
    Trace
}

#[doc(hidden)]
pub fn __private_log(
    core: bool,
    level: LogLevel,
    file: &'static str,
    line: u32,
    args: fmt::Arguments,
) {
    let levelstr = match level {
        LogLevel::Error   => "ERROR".red(),
        LogLevel::Warning => "WARNING".yellow(),
        LogLevel::Info    => "INFO".green(),
        LogLevel::Debug   => "DEBUG".blue(),
        LogLevel::Trace   => "TRACE".truecolor(80, 80, 80),
    };
    let corestr = match core {
        true => " LY".magenta(),
        false => "".normal(),
    };

    unsafe {
        LOGGER.log(format_args!("[{:7}{}] {}:{} {}",
                 levelstr, corestr,
                 file, line,
                 args));
    }
}

trait Log: Sync
{
    fn log(&self, args: fmt::Arguments);
}

struct EmptyLogger;

impl Log for EmptyLogger
{
    fn log(&self, _args: fmt::Arguments) { }
}

static mut LOGGER: &dyn Log = &EmptyLogger;

fn init_channel() -> mpsc::SyncSender<String>
{
    let (tx, rx) = mpsc::sync_channel(5);

    thread::spawn(move || {
        for line in rx {
            println!("{}", line);
        }
    });
    tx
}

pub fn init()
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

struct Logger
{
    transmitter: ThreadLocal<mpsc::SyncSender<String>>,
    tx_main: mpsc::SyncSender<String>,
}

impl Logger
{
    fn new(tx: mpsc::SyncSender<String>) -> Self
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
    fn log(&self, args: fmt::Arguments)
    {
        let tx = self.transmitter.get_or(|| self.tx_main.clone());
        tx.send(format!("{}", args));
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

#[cfg(feature = "warning")]
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

#[cfg(not(feature = "warning"))]
#[macro_export]
macro_rules! warning { ($($x : tt) *) => { } }

#[cfg(feature = "info")]
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

#[cfg(not(feature = "info"))]
#[macro_export]
macro_rules! info { ($($x : tt) *) => { } }


#[cfg(feature = "debug")]
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

#[cfg(not(feature = "debug"))]
#[macro_export]
macro_rules! debug { ($($x : tt) *) => { } }

#[cfg(feature = "trace")]
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

#[cfg(not(feature = "trace"))]
#[macro_export]
macro_rules! trace { ($($x : tt) *) => { } }

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

#[cfg(feature = "warning")]
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

#[cfg(not(feature = "warning"))]
#[macro_export]
macro_rules! core_warning { ($($x : tt) *) => { } }

#[cfg(feature = "info")]
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

#[cfg(not(feature = "info"))]
#[macro_export]
macro_rules! core_info { ($($x : tt) *) => { } }

#[cfg(feature = "debug")]
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

#[cfg(not(feature = "debug"))]
#[macro_export]
macro_rules! core_debug { ($($x : tt) *) => { } }

#[cfg(feature = "trace")]
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

#[cfg(not(feature = "trace"))]
#[macro_export]
macro_rules! core_trace { ($($x : tt) *) => { } }
