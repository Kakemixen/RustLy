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
