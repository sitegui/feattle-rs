mod pages;

use crate::pages::Pages;
use feattle_core::persist::Persist;
use feattle_core::Feattles;
use serde::export::PhantomData;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use warp::http::Uri;
use warp::{Filter, Rejection, Reply};

pub struct AdminPanel<F, P> {
    feattles: Arc<F>,
    pages: Pages,
    _phantom: PhantomData<P>,
}

#[derive(Debug, Deserialize)]
struct EditFeatureForm {
    value_json: String,
}

impl<F: Feattles<P>, P: Persist> AdminPanel<F, P> {
    pub fn new(feattles: Arc<F>, label: String) -> Arc<Self> {
        Arc::new(AdminPanel {
            feattles,
            pages: Pages::new(label),
            _phantom: PhantomData,
        })
    }

    pub fn warp_filter(
        self: Arc<Self>,
    ) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let list_features = {
            let this = self.clone();
            warp::path::end().and(warp::get()).map(move || {
                this.pages
                    .render_features(this.feattles.definitions())
                    .unwrap()
            })
        };

        let show_feature = {
            let this = self.clone();
            warp::path!("feature" / String)
                .and(warp::get())
                .map(move |key: String| {
                    let definition = this.feattles.definition(&key).unwrap();
                    let history = this.feattles.history(&key).unwrap();
                    this.pages.render_feature(&definition, &history).unwrap()
                })
        };

        let edit_feature = {
            let this = self.clone();
            warp::path!("feature" / String / "edit")
                .and(warp::post())
                .and(warp::body::form())
                .map(move |key: String, form: EditFeatureForm| {
                    log::info!(
                        "Received edit request for key {} with value {}",
                        key,
                        form.value_json
                    );
                    let value: Value = serde_json::from_str(&form.value_json).unwrap();
                    this.feattles
                        .update(&key, value, "admin".to_owned())
                        .unwrap();
                    warp::redirect(Uri::from_static("/"))
                })
        };

        let public_files = {
            let this = self.clone();
            warp::path!("public" / String)
                .and(warp::get())
                .map(move |file_name: String| this.pages.render_public_file(&file_name))
        };

        list_features
            .or(show_feature)
            .or(edit_feature)
            .or(public_files)
    }
}
