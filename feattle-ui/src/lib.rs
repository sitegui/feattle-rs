//! This crate implements an administration Web Interface for visualizing and modifying the feature
//! flags (called "feattles", for short).
//!
//! It provides a web-framework-agnostic implementation in [`AdminPanel`] and a ready-to-use binding
//! to `warp` in [`run_warp_server`]. Please refer to the
//! [main package - `feattle`](https://crates.io/crates/feattle) for more information.
//!
//! # Optional features
//!
//! - **warp**: provides [`run_warp_server`] for a read-to-use integration with [`warp`]

mod pages;
#[cfg(feature = "warp")]
mod warp_ui;

use crate::pages::{PageError, Pages};
use feattle_core::persist::Persist;
use feattle_core::{Feattles, HistoryError, UpdateError};
use serde::export::PhantomData;
use serde_json::Value;
use std::error::Error;
use std::sync::Arc;

#[cfg(feature = "warp")]
pub use warp_ui::run_warp_server;

/// The administration panel, agnostic to the choice of web-framework.
///
/// This type is designed to be easily integrated with Rust web-frameworks, by providing one method
/// per page and form submission, each returning bytes with their "Content-Type".
///
/// # Example
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use feattle_ui::AdminPanel;
/// use feattle_core::{feattles, Feattles};
/// use feattle_core::persist::NoPersistence;
/// use std::sync::Arc;
///
/// feattles! {
///     struct MyToggles { a: bool, b: i32 }
/// }
///
/// // `NoPersistence` here is just a mock for the sake of the example
/// let my_toggles = Arc::new(MyToggles::new(NoPersistence));
/// let admin_panel = AdminPanel::new(my_toggles, "Project Panda - DEV".to_owned());
///
/// let home_content = admin_panel.list_feattles()?;
/// assert_eq!(home_content.content_type, "text/html; charset=utf-8");
/// assert!(home_content.content.len() > 0);
/// # Ok(())
/// # }
/// ```
pub struct AdminPanel<F, P> {
    feattles: Arc<F>,
    pages: Pages,
    _phantom: PhantomData<P>,
}

/// Represent a rendered page
#[derive(Debug, Clone)]
pub struct RenderedPage {
    /// The value for the `Content-Type` header
    pub content_type: String,
    /// The response body, as bytes
    pub content: Vec<u8>,
}

/// Represent what can go wrong while handling a request
#[derive(Debug, thiserror::Error)]
pub enum RenderError<PersistError: Error + Send + Sync + 'static> {
    /// The requested page does not exist
    #[error("the requested page does not exist")]
    NotFound,
    /// The template failed to render
    #[error("the template failed to render")]
    Template(#[from] handlebars::RenderError),
    /// Failed to serialize or deserialize JSON
    #[error("failed to serialize or deserialize JSON")]
    Serialization(#[from] serde_json::Error),
    /// Failed to recover history information
    #[error("failed to recover history information")]
    History(#[from] HistoryError<PersistError>),
    /// Failed to update value
    #[error("failed to update value")]
    Update(#[from] UpdateError<PersistError>),
}

impl<PersistError: Error + Send + Sync + 'static> From<PageError> for RenderError<PersistError> {
    fn from(error: PageError) -> Self {
        match error {
            PageError::NotFound => RenderError::NotFound,
            PageError::Template(error) => RenderError::Template(error),
            PageError::Serialization(error) => RenderError::Serialization(error),
        }
    }
}

impl<F: Feattles<P>, P: Persist> AdminPanel<F, P> {
    /// Create a new UI provider for a given feattles and a user-visible label
    pub fn new(feattles: Arc<F>, label: String) -> Self {
        AdminPanel {
            feattles,
            pages: Pages::new(label),
            _phantom: PhantomData,
        }
    }

    /// Render the page that lists the current feattles values, together with navigation links to
    /// modify them. This page is somewhat the "home screen" of the UI.
    pub fn list_feattles(&self) -> Result<RenderedPage, RenderError<P::Error>> {
        Ok(self.pages.render_feattles(
            &self.feattles.definitions(),
            self.feattles.last_reload(),
            self.feattles.current_values().as_deref(),
        )?)
    }

    /// Render the page that shows the current and historical values of a single feattle, together
    /// with the form to modify it. The generated form submits to "/feattle/{{ key }}/edit" with the
    /// POST method in url-encoded format with a single field called "value_json".
    pub async fn show_feattle(&self, key: &str) -> Result<RenderedPage, RenderError<P::Error>> {
        let definition = self
            .feattles
            .definition(&key)
            .ok_or(RenderError::NotFound)?;
        let history = self.feattles.history(&key).await?;
        Ok(self
            .pages
            .render_feattle(&definition, &history, self.feattles.last_reload())?)
    }

    /// Process a modification of a single feattle, given its key and the JSON representation of its
    /// future value. In case of success, the return is empty, so caller should usually redirect the
    /// user somewhere after.
    pub async fn edit_feattle(
        &self,
        key: &str,
        value_json: &str,
    ) -> Result<(), RenderError<P::Error>> {
        log::info!(
            "Received edit request for key {} with value {}",
            key,
            value_json
        );
        let value: Value = serde_json::from_str(&value_json)?;
        self.feattles
            .update(&key, value, "admin".to_owned())
            .await?;
        Ok(())
    }

    /// Renders a public file with the given path. The pages include public files like
    /// "/public/some/path.js", but this method should be called with only the "some/path.js" part.
    pub fn render_public_file(&self, path: &str) -> Result<RenderedPage, RenderError<P::Error>> {
        Ok(self.pages.render_public_file(path)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use feattle_core::{feattles, Feattles};

    feattles! {
        struct MyToggles { a: bool, b: i32 }
    }

    #[tokio::test]
    async fn test() {
        use feattle_core::persist::NoPersistence;

        // `NoPersistence` here is just a mock for the sake of the example
        let my_toggles = Arc::new(MyToggles::new(NoPersistence));
        my_toggles.reload().await.unwrap();
        let admin_panel = Arc::new(AdminPanel::new(
            my_toggles,
            "Project Panda - DEV".to_owned(),
        ));

        // Just check the methods return
        admin_panel.list_feattles().unwrap();
        admin_panel.show_feattle("a").await.unwrap();
        admin_panel.show_feattle("non-existent").await.unwrap_err();
        admin_panel.render_public_file("script.js").unwrap();
        admin_panel.render_public_file("non-existent").unwrap_err();
        admin_panel.edit_feattle("a", "true").await.unwrap();
        admin_panel.edit_feattle("a", "17").await.unwrap_err();
    }
}
