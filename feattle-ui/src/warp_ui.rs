use crate::api::v1;
use crate::{AdminPanel, RenderError, RenderedPage};
use feattle_core::persist::Persist;
use feattle_core::{Feattles, UpdateError};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use warp::filters::path;
use warp::http::{StatusCode, Uri};
use warp::reject::Reject;
use warp::{reject, reply, Filter, Rejection, Reply};

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
    F: Feattles<P> + Sync + Send + 'static,
    P: Persist + Sync + Send + 'static,
{
    let admin_panel = warp::any().map(move || admin_panel.clone());

    let list_feattles = warp::path::end()
        .and(warp::get())
        .and(admin_panel.clone())
        .and_then(|admin_panel: Arc<AdminPanel<F, P>>| async move {
            admin_panel
                .list_feattles()
                .await
                .map_err(to_rejection)
                .map(to_reply)
        });

    let list_feattles_api = warp::path!("feattles")
        .and(warp::get())
        .and(admin_panel.clone())
        .and_then(|admin_panel: Arc<AdminPanel<F, P>>| async move {
            to_json_result(admin_panel.list_feattles_api_v1().await)
        });

    let show_feattle = warp::path!("feattle" / String)
        .and(warp::get())
        .and(admin_panel.clone())
        .and_then(
            |key: String, admin_panel: Arc<AdminPanel<F, P>>| async move {
                admin_panel
                    .show_feattle(&key)
                    .await
                    .map_err(to_rejection)
                    .map(to_reply)
            },
        );

    let show_feattle_api = warp::path!("feattle" / String)
        .and(warp::get())
        .and(admin_panel.clone())
        .and_then(
            |key: String, admin_panel: Arc<AdminPanel<F, P>>| async move {
                to_json_result(admin_panel.show_feattle_api_v1(&key).await)
            },
        );

    let edit_feattle = warp::path!("feattle" / String / "edit")
        .and(warp::post())
        .and(admin_panel.clone())
        .and(warp::body::form())
        .and_then(
            |key: String, admin_panel: Arc<AdminPanel<F, P>>, form: EditFeattleForm| async move {
                admin_panel
                    .edit_feattle(&key, &form.value_json, "admin".to_owned())
                    .await
                    .map_err(to_rejection)
                    .map(|_| warp::redirect(Uri::from_static("/")))
            },
        );

    let edit_feattle_api =
        warp::path!("feattle" / String)
            .and(warp::post())
            .and(admin_panel.clone())
            .and(warp::body::json())
            .and_then(
                |key: String,
                 admin_panel: Arc<AdminPanel<F, P>>,
                 request: v1::EditFeattleRequest| async move {
                    to_json_result(admin_panel.edit_feattle_api_v1(&key, request).await)
                },
            );

    let public_files = warp::path!("public" / String)
        .and(warp::get())
        .and(admin_panel.clone())
        .and_then(
            |file_name: String, admin_panel: Arc<AdminPanel<F, P>>| async move {
                admin_panel
                    .render_public_file(&file_name)
                    .map_err(to_rejection)
                    .map(to_reply)
            },
        );

    let api = path::path("api")
        .and(path::path("v1"))
        .and(list_feattles_api.or(show_feattle_api).or(edit_feattle_api));

    warp::serve(
        list_feattles
            .or(show_feattle)
            .or(edit_feattle)
            .or(public_files)
            .or(api),
    )
    .run(addr)
    .await;
}

impl<PersistError: Error + Send + Sync + 'static> Reject for RequestError<PersistError> {}

fn to_reply(page: RenderedPage) -> impl Reply {
    reply::with_header(page.content, "Content-Type", page.content_type)
}

fn to_rejection<PersistError: Error + Sync + Send + 'static>(
    error: RenderError<PersistError>,
) -> Rejection {
    if let RenderError::NotFound = error {
        reject::not_found()
    } else {
        log::error!("request failed with {:?}", error);
        reject::custom(RequestError(error))
    }
}

fn to_json_result<T: Serialize, PersistError: Error + Sync + Send + 'static>(
    value: Result<T, RenderError<PersistError>>,
) -> Result<Box<dyn Reply>, Rejection> {
    match value {
        Ok(ok) => Ok(Box::new(reply::json(&ok))),
        Err(RenderError::NotFound) | Err(RenderError::Update(UpdateError::UnknownKey(_))) => {
            Ok(Box::new(StatusCode::NOT_FOUND))
        }
        Err(RenderError::Update(UpdateError::Parsing(err))) => Ok(Box::new(reply::with_status(
            format!("Failed to parse: {:?}", err),
            StatusCode::BAD_REQUEST,
        ))),
        Err(err) => Err(reject::custom(RequestError(err))),
    }
}
