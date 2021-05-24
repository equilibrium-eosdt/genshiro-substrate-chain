pub use crate::gens_log;

#[cfg(feature = "std")]
#[macro_export]
macro_rules! gens_log {
  ($($arg:tt)+) => {
    println!($($arg)+);
  }
}

#[cfg(not(feature = "std"))]
#[macro_export]
macro_rules! gens_log {
  ($($arg:tt)+) => {
        debug::warn!($($arg)+)
    }
}
