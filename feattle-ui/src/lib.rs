mod pages;
pub mod warp_ui;

use crate::pages::Pages;
use feattle_core::persist::Persist;
use feattle_core::{Feattles, HistoryError, UpdateError};
use serde::export::PhantomData;
use serde_json::Value;
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
pub enum RenderError {
    #[error("The requested page does not exist")]
    NotFound,
    #[error("The template failed to render")]
    Template(#[from] handlebars::RenderError),
    #[error("Failed to serialize or deserialize JSON")]
    Serialization(#[from] serde_json::Error),
    #[error("Failed to recover history information")]
    History(#[from] HistoryError),
    #[error("Failed to update value")]
    Update(#[from] UpdateError),
}

pub type RenderResult = Result<RenderedPage, RenderError>;

impl<F: Feattles<P>, P: Persist> AdminPanel<F, P> {
    pub fn new(feattles: Arc<F>, label: String) -> Arc<Self> {
        Arc::new(AdminPanel {
            feattles,
            pages: Pages::new(label),
            _phantom: PhantomData,
        })
    }

    pub fn list_features(&self) -> RenderResult {
        self.pages.render_features(self.feattles.definitions())
    }

    pub async fn show_feature(&self, key: &str) -> RenderResult {
        let definition = self
            .feattles
            .definition(&key)
            .ok_or(RenderError::NotFound)?;
        let history = self.feattles.history(&key).await?;
        self.pages.render_feature(&definition, &history)
    }

    pub async fn edit_feature(&self, key: &str, value_json: &str) -> Result<(), RenderError> {
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

    pub fn render_public_file(&self, path: &str) -> RenderResult {
        self.pages.render_public_file(path)
    }
}
