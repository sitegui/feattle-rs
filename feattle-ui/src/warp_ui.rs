use crate::{AdminPanel, RenderError, RenderedPage};
use feattle_core::persist::Persist;
use feattle_core::Feattles;
use serde::Deserialize;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use warp::http::Uri;
use warp::reject::{custom, not_found, Reject};
use warp::reply::with_header;
use warp::{Filter, Rejection, Reply};

#[derive(Debug)]
pub struct RequestError<PersistError: Error + Send + Sync + 'static>(RenderError<PersistError>);

#[derive(Debug, Deserialize)]
struct EditFeatureForm {
    value_json: String,
}

pub async fn run_server<F, P>(
    admin_panel: Arc<AdminPanel<F, P>>,
    addr: impl Into<SocketAddr> + 'static,
) where
    F: Feattles<P>,
    P: Persist,
{
    let list_features = {
        let admin_panel = admin_panel.clone();
        warp::path::end().and(warp::get()).and_then(move || {
            let admin_panel = admin_panel.clone();
            async move {
                admin_panel
                    .list_features()
                    .map_err(to_rejection)
                    .map(to_reply)
            }
        })
    };

    let show_feature = {
        let admin_panel = admin_panel.clone();
        warp::path!("feature" / String)
            .and(warp::get())
            .and_then(move |key: String| {
                let admin_panel = admin_panel.clone();
                async move {
                    admin_panel
                        .show_feature(&key)
                        .await
                        .map_err(to_rejection)
                        .map(to_reply)
                }
            })
    };

    let edit_feature = {
        let admin_panel = admin_panel.clone();
        warp::path!("feature" / String / "edit")
            .and(warp::post())
            .and(warp::body::form())
            .and_then(move |key: String, form: EditFeatureForm| {
                let admin_panel = admin_panel.clone();
                async move {
                    admin_panel
                        .edit_feature(&key, &form.value_json)
                        .await
                        .map_err(to_rejection)
                        .map(|_| warp::redirect(Uri::from_static("/")))
                }
            })
    };

    let public_files = {
        let admin_panel = admin_panel.clone();
        warp::path!("public" / String)
            .and(warp::get())
            .and_then(move |file_name: String| {
                let admin_panel = admin_panel.clone();
                async move {
                    admin_panel
                        .render_public_file(&file_name)
                        .map_err(to_rejection)
                        .map(to_reply)
                }
            })
    };

    warp::serve(
        list_features
            .or(show_feature)
            .or(edit_feature)
            .or(public_files),
    )
    .run(addr)
    .await;
}

impl<PersistError: Error + Send + Sync + 'static> Reject for RequestError<PersistError> {}

fn to_reply(page: RenderedPage) -> impl Reply {
    with_header(page.content, "Content-Type", page.content_type)
}

fn to_rejection<PersistError: Error + Sync + Send + 'static>(
    error: RenderError<PersistError>,
) -> Rejection {
    if let RenderError::NotFound = error {
        not_found()
    } else {
        log::error!("request failed with {:?}", error);
        custom(RequestError(error))
    }
}
