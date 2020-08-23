use chrono::{DateTime, Utc};
use feattle_core::persist::ValueHistory;
use feattle_core::FeatureDefinition;
use handlebars::Handlebars;
use serde_json::json;
use std::collections::BTreeMap;
use std::error::Error;
use std::sync::Arc;
use warp::http::StatusCode;
use warp::reply::{html, Html};
use warp::Reply;

#[derive(Debug, Clone)]
pub struct Pages {
    handlebars: Arc<Handlebars<'static>>,
    public_files: BTreeMap<&'static str, PublicFile>,
    label: String,
}

#[derive(Debug, Clone)]
struct PublicFile {
    content: &'static str,
    content_type: &'static str,
}

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
                content: include_str!("../web/script.js"),
                content_type: "application/javascript",
            },
        );

        Pages {
            handlebars: Arc::new(handlebars),
            public_files,
            label,
        }
    }

    pub fn render_public_file(&self, path: &str) -> Box<dyn Reply> {
        match self.public_files.get(path) {
            None => Box::new(warp::reply::with_status("Not found", StatusCode::NOT_FOUND)),
            Some(file) => Box::new(warp::reply::with_header(
                file.content.to_owned(),
                "Content-Type",
                file.content_type.to_owned(),
            )),
        }
    }

    pub fn render_features(
        &self,
        definitions: Vec<FeatureDefinition>,
    ) -> Result<Html<String>, Box<dyn Error>> {
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

        Ok(html(self.handlebars.render(
            "features",
            &json!({ "features": features, "label": self.label }),
        )?))
    }

    pub fn render_feature(
        &self,
        definition: &FeatureDefinition,
        history: &ValueHistory,
    ) -> Result<Html<String>, Box<dyn Error>> {
        let history = history
            .entries
            .iter()
            .map(|entry| -> Result<_, _> {
                Ok(json!({
                    "modified_at": date_string(entry.modified_at),
                    "modified_by": entry.modified_by,
                    "value_overview": entry.value_overview,
                    "value_json": serde_json::to_string(&entry.value)?,
                }))
            })
            .collect::<Result<Vec<_>, Box<dyn Error>>>()?;

        Ok(html(self.handlebars.render(
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
        )?))
    }
}

fn last_modification(definition: &FeatureDefinition) -> String {
    match (&definition.modified_at, &definition.modified_by) {
        (&Some(at), Some(by)) => format!("{} by {}", date_string(at), by),
        _ => "unknown".to_owned(),
    }
}

fn date_string(datetime: DateTime<Utc>) -> String {
    datetime.format("%Y-%m-%d %H:%M:%S %Z").to_string()
}
