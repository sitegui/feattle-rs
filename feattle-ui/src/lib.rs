use feattle_core::Feattles;
use std::sync::Arc;
use warp::{Filter, Rejection, Reply};

pub fn ui(
    feattles: Arc<impl Feattles>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let home = {
        let feattles = feattles.clone();
        warp::path::end().map(move || format!("{:#?}", feattles.definitions()))
    };
    let feature = {
        let feattles = feattles.clone();
        warp::path!("feature" / String)
            .map(move |key: String| format!("{:#?}", feattles.definition(&key)))
    };

    home.or(feature)
}
