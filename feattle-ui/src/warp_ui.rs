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
struct RequestError<PersistError: Error + Send + Sync + 'static>(RenderError<PersistError>);

#[derive(Debug, Deserialize)]
struct EditFeattleForm {
    value_json: String,
}

/// Run the given admin panel using [`warp`] framework.
///
/// To use it, make sure to activate the cargo feature `"warp"` in your `Cargo.toml`.
///
/// # Example
/// ```no_run
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use feattle_ui::{AdminPanel, run_warp_server};
/// use feattle_core::{feattles, Feattles};
/// use feattle_core::persist::NoPersistence;
/// use std::sync::Arc;
///
/// feattles! {
///     struct MyToggles { a: bool, b: i32 }
/// }
///
/// // `NoPersistence` here is just a mock for the sake of the example
/// let my_toggles = Arc::new(MyToggles::new(NoPersistence));
/// let admin_panel = Arc::new(AdminPanel::new(my_toggles, "Project Panda - DEV".to_owned()));
///
/// run_warp_server(admin_panel, ([127, 0, 0, 1], 3030)).await;
/// # Ok(())
/// # }
/// ```
pub async fn run_warp_server<F, P>(
    admin_panel: Arc<AdminPanel<F, P>>,
    addr: impl Into<SocketAddr> + 'static,
) where
    F: Feattles<P>,
    P: Persist,
{
    let list_feattles = {
        let admin_panel = admin_panel.clone();
        warp::path::end().and(warp::get()).and_then(move || {
            let admin_panel = admin_panel.clone();
            async move {
                admin_panel
                    .list_feattles()
                    .map_err(to_rejection)
                    .map(to_reply)
            }
        })
    };

    let show_feattle = {
        let admin_panel = admin_panel.clone();
        warp::path!("feattle" / String)
            .and(warp::get())
            .and_then(move |key: String| {
                let admin_panel = admin_panel.clone();
                async move {
                    admin_panel
                        .show_feattle(&key)
                        .await
                        .map_err(to_rejection)
                        .map(to_reply)
                }
            })
    };

    let edit_feattle = {
        let admin_panel = admin_panel.clone();
        warp::path!("feattle" / String / "edit")
            .and(warp::post())
            .and(warp::body::form())
            .and_then(move |key: String, form: EditFeattleForm| {
                let admin_panel = admin_panel.clone();
                async move {
                    admin_panel
                        .edit_feattle(&key, &form.value_json)
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
        list_feattles
            .or(show_feattle)
            .or(edit_feattle)
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
