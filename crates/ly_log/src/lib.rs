pub use colored::Colorize;
use std::fmt;

pub mod prelude
{
    pub use super::{LogLevel, __private_log,
        error, warning, info, debug, trace};
}

pub mod core_prelude
{
    pub use super::{LogLevel, __private_log, core_error,
        core_warning, core_info, core_debug, core_trace};
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
    println!("[{:7}{}] {}:{} {}",
             levelstr, corestr,
             file!(), line!(),
             args);
}

#[macro_export]
macro_rules! error
{
    () => { };
    ($($x : tt) *) => { __private_log(
            false,
            LogLevel::Error,
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
    ($($x : tt) *) => { __private_log(
            false,
            LogLevel::Warning,
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
    ($($x : tt) *) => { __private_log(
            false,
            LogLevel::Info,
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
    ($($x : tt) *) => { __private_log(
            false,
            LogLevel::Debug,
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
    ($($x : tt) *) => { __private_log(
            false,
            LogLevel::Trace,
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
    ($($x : tt) *) => { __private_log(
            true,
            LogLevel::Error,
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
    ($($x : tt) *) => { __private_log(
            true,
            LogLevel::Warning,
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
    ($($x : tt) *) => { __private_log(
            true,
            LogLevel::Info,
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
    ($($x : tt) *) => { __private_log(
            true,
            LogLevel::Debug,
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
    ($($x : tt) *) => { __private_log(
            true,
            LogLevel::Trace,
            format_args!(
                $($x) *
                )
            ) };
}

#[cfg(not(feature = "trace"))]
#[macro_export]
macro_rules! core_trace { ($($x : tt) *) => { } }
