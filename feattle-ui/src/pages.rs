use crate::RenderedPage;
use chrono::{DateTime, Utc};
use feattle_core::last_reload::LastReload;
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
        macro_rules! register_template {
            ($name:expr) => {
                handlebars
                    .register_template_string(
                        $name,
                        include_str!(concat!("../web/", $name, ".hbs")),
                    )
                    .expect("The handlebars template should be valid");
            };
        }
        register_template!("layout");
        register_template!("feattles");
        register_template!("feattle");

        let mut public_files = BTreeMap::new();
        macro_rules! insert_public_file {
            ($name:expr, $content_type:expr) => {
                public_files.insert(
                    $name,
                    PublicFile {
                        content: include_bytes!(concat!("../web/", $name)),
                        content_type: $content_type,
                    },
                );
            };
        }
        insert_public_file!("script.js", "application/javascript");
        insert_public_file!("style.css", "text/css");
        insert_public_file!("favicon-32x32.png", "image/png");

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

    pub fn render_feattles(
        &self,
        definitions: &[FeattleDefinition],
        last_reload: LastReload,
        reload_failed: bool,
    ) -> PageResult {
        let feattles: Vec<_> = definitions
            .iter()
            .map(|definition| {
                json!({
                    "key": definition.key,
                    "format": definition.format.tag,
                    "description": definition.description,
                    "value_overview": definition.value_overview,
                    "last_modification": last_modification(definition, last_reload),
                })
            })
            .collect();
        let version = match last_reload {
            LastReload::Never | LastReload::NoData { .. } => "unknown".to_owned(),
            LastReload::Data {
                version,
                version_date,
                ..
            } => format!("{}, created at {}", version, date_string(version_date)),
        };
        let last_reload_str = match last_reload.reload_date() {
            None => "never".to_owned(),
            Some(date) => date_string(date),
        };

        Self::convert_html(self.handlebars.render(
            "feattles",
            &json!({
                 "feattles": feattles,
                 "label": self.label,
                 "last_reload": last_reload_str,
                 "version": version,
                 "reload_failed": reload_failed,
            }),
        ))
    }

    pub fn render_feattle(
        &self,
        definition: &FeattleDefinition,
        history: &ValueHistory,
        last_reload: LastReload,
        reload_failed: bool,
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
            "feattle",
            &json!({
                "key": definition.key,
                "format": definition.format.tag,
                "description": definition.description,
                "value_overview": definition.value_overview,
                "last_modification": last_modification(definition, last_reload),
                "format_json": serde_json::to_string(&definition.format.kind)?,
                "value_json": serde_json::to_string(&definition.value)?,
                "label": self.label,
                "history": history,
                "reload_failed": reload_failed,
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

fn last_modification(definition: &FeattleDefinition, last_reload: LastReload) -> String {
    match (last_reload, definition.modified_at, &definition.modified_by) {
        (LastReload::Never, _, _) => "unknown".to_owned(),
        (LastReload::NoData { .. }, _, _)
        | (LastReload::Data { .. }, None, _)
        | (LastReload::Data { .. }, _, None) => "never".to_owned(),
        (LastReload::Data { .. }, Some(at), Some(by)) => {
            format!("{} by {}", date_string(at), by)
        }
    }
}

fn date_string(datetime: DateTime<Utc>) -> String {
    datetime.format("%Y-%m-%d %H:%M:%S %Z").to_string()
}
