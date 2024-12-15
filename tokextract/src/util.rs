//! Utilities for [`fantoccini`].

use std::time::Duration;

use fantoccini::{
    actions::{InputSource, KeyAction, KeyActions},
    elements::Element,
    error::CmdError,
    key::Key,
};

use crate::private::Sealed;

/// Utilities for [`Element`].
pub trait ElementExt: Sealed {
    /// Scrolls the element into view.
    async fn scroll_into_view(&self) -> Result<(), CmdError>;
    /// Presses and releases a key.
    async fn send_key(&self, key: Key) -> Result<(), CmdError>;
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

    async fn send_key(&self, key: Key) -> Result<(), CmdError> {
        self.clone()
            .client()
            .perform_actions(
                KeyActions::new("keypress".to_string())
                    .then(KeyAction::Down { value: key.into() })
                    .then(KeyAction::Pause {
                        duration: Duration::from_millis(50),
                    })
                    .then(KeyAction::Up { value: key.into() }),
            )
            .await
    }
}
