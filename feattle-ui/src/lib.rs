mod pages;

use crate::pages::Pages;
use feattle_core::persist::Persist;
use feattle_core::Feattles;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use warp::http::Uri;
use warp::{Filter, Rejection, Reply};

#[derive(Debug, Deserialize)]
struct EditFeatureForm {
    value_json: String,
}

pub fn ui<P: Persist>(
    feattles: Arc<impl Feattles<P>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let pages = Pages::new().unwrap();

    let list_features = {
        let feattles = feattles.clone();
        let pages = pages.clone();
        warp::path::end()
            .and(warp::get())
            .map(move || pages.render_features(feattles.definitions()).unwrap())
    };

    let show_feature = {
        let feattles = feattles.clone();
        warp::path!("feature" / String)
            .and(warp::get())
            .map(move |key: String| {
                pages
                    .render_feature(feattles.definition(&key).unwrap())
                    .unwrap()
            })
    };

    let edit_feature = {
        let feattles = feattles.clone();
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
                feattles.update(&key, value, "admin".to_owned()).unwrap();
                warp::redirect(Uri::from_static("/"))
            })
    };

    list_features.or(show_feature).or(edit_feature)
}
