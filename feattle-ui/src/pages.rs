use crate::RenderedPage;
use chrono::{DateTime, Utc};
use feattle_core::persist::ValueHistory;
use feattle_core::FeattleDefinition;
use handlebars::Handlebars;
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Pages {
    handlebars: Arc<Handlebars<'static>>,
    public_files: BTreeMap<&'static str, PublicFile>,
    label: String,
}

#[derive(Debug, thiserror::Error)]
pub enum PageError {
    #[error("The requested page does not exist")]
    NotFound,
    #[error("The template failed to render")]
    Template(#[from] handlebars::RenderError),
    #[error("Failed to serialize or deserialize JSON")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone)]
struct PublicFile {
    content: &'static [u8],
    content_type: &'static str,
}

pub type PageResult = Result<RenderedPage, PageError>;

impl Pages {
    pub fn new(label: String) -> Self {
        let mut handlebars = Handlebars::new();

        handlebars
            .register_template_string("layout", include_str!("../web/layout.hbs"))
            .expect("The handlebars template should be valid");
        handlebars
            .register_template_string("features", include_str!("../web/features.hbs"))
            .expect("The handlebars template should be valid");
        handlebars
            .register_template_string("feature", include_str!("../web/feature.hbs"))
            .expect("The handlebars template should be valid");

        let mut public_files = BTreeMap::new();
        public_files.insert(
            "script.js",
            PublicFile {
                content: include_bytes!("../web/script.js"),
                content_type: "application/javascript",
            },
        );
        public_files.insert(
            "favicon-32x32.png",
            PublicFile {
                content: include_bytes!("../web/favicon-32x32.png"),
                content_type: "image/png",
            },
        );

        Pages {
            handlebars: Arc::new(handlebars),
            public_files,
            label,
        }
    }

    pub fn render_public_file(&self, path: &str) -> PageResult {
        let file = self.public_files.get(path).ok_or(PageError::NotFound)?;
        Ok(RenderedPage {
            content_type: file.content_type.to_owned(),
            content: file.content.to_owned(),
        })
    }

    pub fn render_features(&self, definitions: Vec<FeattleDefinition>) -> PageResult {
        let features: Vec<_> = definitions
            .into_iter()
            .map(|definition| {
                json!({
                    "key": definition.key,
                    "format": definition.format.tag,
                    "description": definition.description,
                    "value_overview": definition.value_overview,
                    "last_modification": last_modification(&definition),
                })
            })
            .collect();

        Self::convert_html(self.handlebars.render(
            "features",
            &json!({ "features": features, "label": self.label }),
        ))
    }

    pub fn render_feature(
        &self,
        definition: &FeattleDefinition,
        history: &ValueHistory,
    ) -> PageResult {
        let history = history
            .entries
            .iter()
            .map(|entry| -> Result<_, PageError> {
                Ok(json!({
                    "modified_at": date_string(entry.modified_at),
                    "modified_by": entry.modified_by,
                    "value_overview": entry.value_overview,
                    "value_json": serde_json::to_string(&entry.value)?,
                }))
            })
            .collect::<Result<Vec<_>, _>>()?;

        Self::convert_html(self.handlebars.render(
            "feature",
            &json!({
                "key": definition.key,
                "format": definition.format.tag,
                "description": definition.description,
                "value_overview": definition.value_overview,
                "last_modification": last_modification(&definition),
                "format_json": serde_json::to_string(&definition.format.kind)?,
                "value_json": serde_json::to_string(&definition.value)?,
                "label": self.label,
                "history": history,
            }),
        ))
    }

    fn convert_html(rendered: Result<String, handlebars::RenderError>) -> PageResult {
        let content = rendered?;
        Ok(RenderedPage {
            content_type: "text/html; charset=utf-8".to_owned(),
            content: content.into_bytes(),
        })
    }
}

fn last_modification(definition: &FeattleDefinition) -> String {
    match (&definition.modified_at, &definition.modified_by) {
        (&Some(at), Some(by)) => format!("{} by {}", date_string(at), by),
        _ => "unknown".to_owned(),
    }
}

fn date_string(datetime: DateTime<Utc>) -> String {
    datetime.format("%Y-%m-%d %H:%M:%S %Z").to_string()
}
