#[macro_export]
macro_rules! error
{
    () => { };
    ($($x : tt) *) => { println!(
            "[ERROR] {}:{} {}",
            file!(), line!(),
            format!(
                $($x) *
                )
            ) };
}

#[cfg(feature = "warning")]
#[macro_export]
macro_rules! warning
{
    () => { };
    ($($x : tt) *) => { println!(
            "[WARNING] {}:{} {}",
            file!(), line!(),
            format!(
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
    ($($x : tt) *) => { println!(
            "[INFO] {}:{} {}",
            file!(), line!(),
            format!(
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
    ($($x : tt) *) => { println!(
            "[DEBUG] {}:{} {}",
            file!(), line!(),
            format!(
                $($x) *
                )
            ) };
}

#[cfg(not(feature = "debug"))]
#[macro_export]
macro_rules! debug { ($($x : tt) *) => { } }

pub fn test_log()
{
    error!("test {}", 2);
    warning!("test {}", 2);
    info!("test {}", 2);
    debug!("test {}", 2);
}
