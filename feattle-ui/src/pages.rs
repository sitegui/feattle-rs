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
        definition: FeatureDefinition,
    ) -> Result<Html<String>, Box<dyn Error>> {
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
                "label": self.label
            }),
        )?))
    }
}

fn last_modification(definition: &FeatureDefinition) -> String {
    match (&definition.modified_at, &definition.modified_by) {
        (Some(at), Some(by)) => format!("{} by {}", at.format("%Y-%m-%d %H:%M:%S"), by),
        _ => "unknown".to_owned(),
    }
}
