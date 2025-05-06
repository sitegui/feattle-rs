use crate::api::v1;
use crate::{AdminPanel, RenderError, RenderedPage};
use async_trait::async_trait;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::{routing, Form, Json, Router};
use feattle_core::{Feattles, UpdateError};
use serde::Deserialize;
use std::sync::Arc;

/// A trait that can be used to extract information about the user that is modifying a feattle.
///
/// If a `Response` is returned, the feattle will not be modified and the given response will be
/// returned.
///
/// For convenience, this trait is implemented for:
/// - strings (`String` and `&'static str`) if simply want to label all modifications with a single
///   name.
/// - functions that take a [`HeaderMap`] and return `Result<String, impl IntoResponse>` if async is
///   not necessary
///
/// For example, to extract the username from a trusted header:
/// ```
/// use axum::http::{HeaderMap, StatusCode};
/// use axum::response::Response;
/// use feattle_ui::axum_router;
/// # let admin_panel = todo!();
///
/// fn get_user(headers: &HeaderMap) -> Result<String, StatusCode> {
///     headers
///         .get("X-User")
///         .and_then(|user| user.to_str().ok())
///         .map(|user| user.to_string())
///         .ok_or(StatusCode::UNAUTHORIZED)
/// }
///
/// let router = axum_router(admin_panel, get_user);
/// ```
#[async_trait]
pub trait ExtractModifiedBy: Send + Sync + 'static {
    async fn extract_modified_by(&self, headers: &HeaderMap) -> Result<String, Response>;
}

/// Return an [`axum`] router that serves the admin panel.
///
/// To use it, make sure to activate the cargo feature `"axum"` in your `Cargo.toml`.
///
/// The router will answer to the web UI under "/" and a JSON API under "/api/v1/" (see more at [`v1`]):
/// - GET /api/v1/feattles
/// - GET /api/v1/feattle/{key}
/// - POST /api/v1/feattle/{key}
///
/// # Example
/// ```no_run
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use std::future::IntoFuture;
/// use feattle_ui::{AdminPanel, axum_router};
/// use feattle_core::{feattles, Feattles};
/// use feattle_core::persist::NoPersistence;
/// use std::sync::Arc;
///
/// use tokio::net::TcpListener;
///
/// feattles! {
///     struct MyToggles { a: bool, b: i32 }
/// }
///
/// // `NoPersistence` here is just a mock for the sake of the example
/// let my_toggles = Arc::new(MyToggles::new(Arc::new(NoPersistence)));
/// let admin_panel = Arc::new(AdminPanel::new(my_toggles, "Project Panda - DEV".to_owned()));
///
/// let router = axum_router(admin_panel, "admin");
///
/// let listener = TcpListener::bind(("127.0.0.1", 3031)).await?;
/// tokio::spawn(axum::serve(listener, router.into_make_service()).into_future());
///
/// # Ok(())
/// # }
/// ```
pub fn axum_router<F>(
    admin_panel: Arc<AdminPanel<F>>,
    extract_modified_by: impl ExtractModifiedBy,
) -> Router<()>
where
    F: Feattles + Sync + Send + 'static,
{
    async fn list_feattles<F: Feattles + Sync>(
        State(state): State<RouterState<F>>,
    ) -> impl IntoResponse {
        state.admin_panel.list_feattles().await
    }

    async fn list_feattles_api_v1<F: Feattles + Sync>(
        State(state): State<RouterState<F>>,
    ) -> impl IntoResponse {
        state.admin_panel.list_feattles_api_v1().await.map(Json)
    }

    async fn show_feattle<F: Feattles + Sync>(
        State(state): State<RouterState<F>>,
        Path(key): Path<String>,
    ) -> impl IntoResponse {
        state.admin_panel.show_feattle(&key).await
    }

    async fn show_feattle_api_v1<F: Feattles + Sync>(
        State(state): State<RouterState<F>>,
        Path(key): Path<String>,
    ) -> impl IntoResponse {
        state.admin_panel.show_feattle_api_v1(&key).await.map(Json)
    }

    async fn edit_feattle<F: Feattles + Sync>(
        State(state): State<RouterState<F>>,
        Path(key): Path<String>,
        headers: HeaderMap,
        Form(form): Form<EditFeattleForm>,
    ) -> Response {
        let modified_by = match state
            .extract_modified_by
            .extract_modified_by(&headers)
            .await
        {
            Ok(modified_by) => modified_by,
            Err(response) => return response,
        };

        state
            .admin_panel
            .edit_feattle(&key, &form.value_json, modified_by)
            .await
            .map(|_| Redirect::to("/"))
            .into_response()
    }

    async fn edit_feattle_api_v1<F: Feattles + Sync>(
        State(state): State<RouterState<F>>,
        Path(key): Path<String>,
        Json(request): Json<v1::EditFeattleRequest>,
    ) -> impl IntoResponse {
        state
            .admin_panel
            .edit_feattle_api_v1(&key, request)
            .await
            .map(Json)
    }

    async fn render_public_file<F: Feattles + Sync>(
        State(state): State<RouterState<F>>,
        Path(file_name): Path<String>,
    ) -> impl IntoResponse {
        state.admin_panel.render_public_file(&file_name)
    }

    let state = RouterState {
        admin_panel,
        extract_modified_by: Arc::new(extract_modified_by),
    };

    Router::new()
        .route("/", routing::get(list_feattles))
        .route("/api/v1/feattles", routing::get(list_feattles_api_v1))
        .route("/feattle/{key}", routing::get(show_feattle))
        .route("/api/v1/feattle/{key}", routing::get(show_feattle_api_v1))
        .route("/feattle/{key}/edit", routing::post(edit_feattle))
        .route("/api/v1/feattle/{key}", routing::post(edit_feattle_api_v1))
        .route("/public/{file_name}", routing::get(render_public_file))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
struct EditFeattleForm {
    value_json: String,
}

struct RouterState<F> {
    admin_panel: Arc<AdminPanel<F>>,
    extract_modified_by: Arc<dyn ExtractModifiedBy>,
}

impl IntoResponse for RenderedPage {
    fn into_response(self) -> Response {
        ([("Content-Type", self.content_type)], self.content).into_response()
    }
}

impl IntoResponse for RenderError {
    fn into_response(self) -> Response {
        match self {
            RenderError::NotFound | RenderError::Update(UpdateError::UnknownKey(_)) => {
                StatusCode::NOT_FOUND.into_response()
            }
            RenderError::Update(UpdateError::Parsing(err)) => (
                StatusCode::BAD_REQUEST,
                format!("Failed to parse: {:?}", err),
            )
                .into_response(),
            err => {
                log::error!("request failed with {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", err)).into_response()
            }
        }
    }
}

impl<F> Clone for RouterState<F> {
    fn clone(&self) -> Self {
        RouterState {
            admin_panel: self.admin_panel.clone(),
            extract_modified_by: self.extract_modified_by.clone(),
        }
    }
}

#[async_trait]
impl ExtractModifiedBy for String {
    async fn extract_modified_by(&self, _headers: &HeaderMap) -> Result<String, Response> {
        Ok(self.clone())
    }
}

#[async_trait]
impl ExtractModifiedBy for &'static str {
    async fn extract_modified_by(&self, _headers: &HeaderMap) -> Result<String, Response> {
        Ok(self.to_string())
    }
}

#[async_trait]
impl<F, R> ExtractModifiedBy for F
where
    F: Fn(&HeaderMap) -> Result<String, R> + Send + Sync + 'static,
    R: IntoResponse,
{
    async fn extract_modified_by(&self, headers: &HeaderMap) -> Result<String, Response> {
        self(headers).map_err(|response| response.into_response())
    }
}
