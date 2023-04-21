//! This crate implements an administration Web Interface for visualizing and modifying the feature
//! flags (called "feattles", for short).
//!
//! It provides a web-framework-agnostic implementation in [`AdminPanel`] and ready-to-use bindings
//! for `warp` and `axum`. Please refer to the
//! [main package - `feattle`](https://crates.io/crates/feattle) for more information.
//!
//! Note that authentication is **not** provided out-of-the-box and you're the one responsible for
//! controlling and protecting the access.
//!
//! # Optional features
//!
//! - **axum**: provides [`axum_router`] for a read-to-use integration with [`axum`]
//! - **warp**: provides [`run_warp_server`] for a read-to-use integration with [`warp`]

pub mod api;
#[cfg(feature = "axum")]
mod axum_ui;
mod pages;
#[cfg(feature = "warp")]
mod warp_ui;

use crate::pages::{PageError, Pages};
use feattle_core::persist::Persist;
use feattle_core::{Feattles, HistoryError, UpdateError};
use serde_json::Value;
use std::error::Error;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::api::v1;
#[cfg(feature = "axum")]
pub use axum_ui::axum_router;
#[cfg(feature = "warp")]
pub use warp_ui::run_warp_server;

/// The administration panel, agnostic to the choice of web-framework.
///
/// This type is designed to be easily integrated with Rust web-frameworks, by providing one method
/// per page and form submission, each returning bytes with their "Content-Type".
///
/// # Example
/// ```
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
/// let home_content = admin_panel.list_feattles().await?;
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
    /// Failed to reload new version
    #[error("failed to reload new version")]
    Reload(#[source] PersistError),
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

impl<F: Feattles<P> + Sync, P: Persist + Sync + 'static> AdminPanel<F, P> {
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
    ///
    /// To ensure fresh data is displayed, [`Feattles::reload()`] is called.
    pub async fn list_feattles(&self) -> Result<RenderedPage, RenderError<P::Error>> {
        let data = self.list_feattles_api_v1().await?;
        Ok(self
            .pages
            .render_feattles(&data.definitions, data.last_reload, data.reload_failed)?)
    }

    /// The JSON-API equivalent of [`AdminPanel::list_feattles()`].
    ///
    /// To ensure fresh data is displayed, [`Feattles::reload()`] is called.
    pub async fn list_feattles_api_v1(
        &self,
    ) -> Result<v1::ListFeattlesResponse, RenderError<P::Error>> {
        let reload_failed = self.feattles.reload().await.is_err();
        Ok(v1::ListFeattlesResponse {
            definitions: self.feattles.definitions(),
            last_reload: self.feattles.last_reload(),
            reload_failed,
        })
    }

    /// Render the page that shows the current and historical values of a single feattle, together
    /// with the form to modify it. The generated form submits to "/feattle/{{ key }}/edit" with the
    /// POST method in url-encoded format with a single field called "value_json".
    ///
    /// To ensure fresh data is displayed, [`Feattles::reload()`] is called.
    pub async fn show_feattle(&self, key: &str) -> Result<RenderedPage, RenderError<P::Error>> {
        let data = self.show_feattle_api_v1(key).await?;
        Ok(self.pages.render_feattle(
            &data.definition,
            &data.history,
            data.last_reload,
            data.reload_failed,
        )?)
    }

    /// The JSON-API equivalent of [`AdminPanel::show_feattle()`].
    ///
    /// To ensure fresh data is displayed, [`Feattles::reload()`] is called.
    pub async fn show_feattle_api_v1(
        &self,
        key: &str,
    ) -> Result<v1::ShowFeattleResponse, RenderError<P::Error>> {
        let reload_failed = self.feattles.reload().await.is_err();
        let definition = self.feattles.definition(key).ok_or(RenderError::NotFound)?;
        let history = self.feattles.history(key).await?;
        Ok(v1::ShowFeattleResponse {
            definition,
            history,
            last_reload: self.feattles.last_reload(),
            reload_failed,
        })
    }

    /// Process a modification of a single feattle, given its key and the JSON representation of its
    /// future value. In case of success, the return is empty, so caller should usually redirect the
    /// user somewhere after.
    ///
    /// To ensure fresh data is displayed, [`Feattles::reload()`] is called. Unlike the other pages,
    /// if the reload fails, this operation will fail.
    pub async fn edit_feattle(
        &self,
        key: &str,
        value_json: &str,
        modified_by: String,
    ) -> Result<(), RenderError<P::Error>> {
        let value: Value = serde_json::from_str(value_json)?;
        self.edit_feattle_api_v1(key, v1::EditFeattleRequest { value, modified_by })
            .await?;
        Ok(())
    }

    /// The JSON-API equivalent of [`AdminPanel::edit_feattle()`].
    ///
    /// To ensure fresh data is displayed, [`Feattles::reload()`] is called. Unlike the other pages,
    /// if the reload fails, this operation will fail.
    pub async fn edit_feattle_api_v1(
        &self,
        key: &str,
        request: v1::EditFeattleRequest,
    ) -> Result<v1::EditFeattleResponse, RenderError<P::Error>> {
        log::info!(
            "Received edit request for key {} with value {}",
            key,
            request.value
        );
        self.feattles.reload().await.map_err(RenderError::Reload)?;
        self.feattles
            .update(key, request.value, request.modified_by)
            .await?;
        Ok(v1::EditFeattleResponse {})
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
        admin_panel.list_feattles().await.unwrap();
        admin_panel.show_feattle("a").await.unwrap();
        admin_panel.show_feattle("non-existent").await.unwrap_err();
        admin_panel.render_public_file("script.js").unwrap();
        admin_panel.render_public_file("non-existent").unwrap_err();
        admin_panel
            .edit_feattle("a", "true", "user".to_owned())
            .await
            .unwrap();
        admin_panel
            .edit_feattle("a", "17", "user".to_owned())
            .await
            .unwrap_err();
    }
}
