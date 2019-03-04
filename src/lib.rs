#[macro_use]
extern crate lazy_static;

#[cfg(feature = "json")]
pub mod spdx_json;
#[cfg(feature = "text")]
pub mod spdx_text;
