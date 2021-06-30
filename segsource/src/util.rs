#[cfg(feature = "async")]
mod sync;
#[cfg(feature = "async")]
pub use sync::*;
