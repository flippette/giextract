//! Utilities for [`fantoccini`].

use fantoccini::{elements::Element, error::CmdError};

use crate::private::Sealed;

/// Utilities for [`Element`].
pub trait ElementExt: Sealed {
    /// Scrolls the element into view.
    async fn scroll_into_view(&self) -> Result<(), CmdError>;
}

impl Sealed for Element {}

impl ElementExt for Element {
    async fn scroll_into_view(&self) -> Result<(), CmdError> {
        const JS: &str = "arguments[0].scrollIntoView();";

        self.clone()
            .client()
            .execute(JS, vec![serde_json::to_value(self)?])
            .await
            .map(drop)
    }
}
