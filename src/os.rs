#[macro_use]
#[cfg(feature = "win32")]
mod win32;
#[cfg(feature = "win32")]
pub use win32::*;

#[cfg(feature = "std")]
mod rust_std;
#[cfg(feature = "std")]
pub use rust_std::*;

