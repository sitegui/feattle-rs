mod pages;
pub mod warp_ui;

use crate::pages::{PageError, Pages};
use feattle_core::persist::Persist;
use feattle_core::{Feattles, HistoryError, UpdateError};
use serde::export::PhantomData;
use serde_json::Value;
use std::error::Error;
use std::sync::Arc;

pub struct AdminPanel<F, P> {
    feattles: Arc<F>,
    pages: Pages,
    _phantom: PhantomData<P>,
}

#[derive(Debug, Clone)]
pub struct RenderedPage {
    content_type: String,
    content: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum RenderError<PersistError: Error + Send + Sync + 'static> {
    #[error("The requested page does not exist")]
    NotFound,
    #[error("The template failed to render")]
    Template(#[from] handlebars::RenderError),
    #[error("Failed to serialize or deserialize JSON")]
    Serialization(#[from] serde_json::Error),
    #[error("Failed to recover history information")]
    History(#[from] HistoryError<PersistError>),
    #[error("Failed to update value")]
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

pub type RenderResult<PersistError> = Result<RenderedPage, RenderError<PersistError>>;

impl<F: Feattles<P>, P: Persist> AdminPanel<F, P> {
    pub fn new(feattles: Arc<F>, label: String) -> Arc<Self> {
        Arc::new(AdminPanel {
            feattles,
            pages: Pages::new(label),
            _phantom: PhantomData,
        })
    }

    pub fn list_features(&self) -> RenderResult<P::Error> {
        Ok(self.pages.render_features(self.feattles.definitions())?)
    }

    pub async fn show_feature(&self, key: &str) -> RenderResult<P::Error> {
        let definition = self
            .feattles
            .definition(&key)
            .ok_or(RenderError::NotFound)?;
        let history = self.feattles.history(&key).await?;
        Ok(self.pages.render_feature(&definition, &history)?)
    }

    pub async fn edit_feature(
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

    pub fn render_public_file(&self, path: &str) -> RenderResult<P::Error> {
        Ok(self.pages.render_public_file(path)?)
    }
}
