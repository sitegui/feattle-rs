use feattle_core::Feattles;
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use warp::{Filter, Rejection, Reply};

#[derive(Debug, Deserialize)]
struct EditFeatureForm {
    value: Value,
}

pub fn ui<P>(
    feattles: Arc<impl Feattles<P>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let list_features = {
        let feattles = feattles.clone();
        warp::path::end()
            .and(warp::get())
            .map(move || format!("{:#?}", feattles.definitions()))
    };
    let show_feature = {
        let feattles = feattles.clone();
        warp::path!("feature" / String)
            .and(warp::get())
            .map(move |key: String| format!("{:#?}", feattles.definition(&key)))
    };
    let edit_feature = {
        let feattles = feattles.clone();
        warp::path!("feature" / String / "edit")
            .and(warp::post())
            .and(warp::body::form())
            .map(move |key: String, _x: EditFeatureForm| {
                format!("{:#?}", feattles.definition(&key))
            })
    };

    list_features.or(show_feature).or(edit_feature)
}
