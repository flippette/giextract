//! GEL IELTS API token extraction.
//!
//! This module interacts with a browser through the WebDriver API to extract
//! the cookie containing the API token.

#![allow(async_fn_in_trait)]

mod server;
mod util;

mod private {
    pub trait Sealed {}
}

pub use server::Server;
pub use util::ElementExt;
