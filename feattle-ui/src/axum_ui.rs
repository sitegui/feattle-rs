use crate::api::v1;
use crate::{AdminPanel, RenderError, RenderedPage};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect, Response};
use axum::{routing, Form, Json, Router};
use feattle_core::{Feattles, UpdateError};
use serde::Deserialize;
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct EditFeattleForm {
    value_json: String,
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
/// use feattle_ui::{AdminPanel, axum_router};
/// use feattle_core::{feattles, Feattles};
/// use feattle_core::persist::NoPersistence;
/// use std::sync::Arc;
/// use axum::Server;
///
/// feattles! {
///     struct MyToggles { a: bool, b: i32 }
/// }
///
/// // `NoPersistence` here is just a mock for the sake of the example
/// let my_toggles = Arc::new(MyToggles::new(NoPersistence));
/// let admin_panel = Arc::new(AdminPanel::new(my_toggles, "Project Panda - DEV".to_owned()));
///
/// let router = axum_router(admin_panel);
///
/// Server::bind(&([127, 0, 0, 1], 3030).into())
///     .serve(router.into_make_service())
///     .await?;
///
/// # Ok(())
/// # }
/// ```
pub fn axum_router<F>(admin_panel: Arc<AdminPanel<F>>) -> Router<()>
where
    F: Feattles + Sync + Send + 'static,
{
    async fn list_feattles<F: Feattles + Sync>(
        State(admin_panel): State<Arc<AdminPanel<F>>>,
    ) -> impl IntoResponse {
        admin_panel.list_feattles().await
    }

    async fn list_feattles_api_v1<F: Feattles + Sync>(
        State(admin_panel): State<Arc<AdminPanel<F>>>,
    ) -> impl IntoResponse {
        admin_panel.list_feattles_api_v1().await.map(Json)
    }

    async fn show_feattle<F: Feattles + Sync>(
        State(admin_panel): State<Arc<AdminPanel<F>>>,
        Path(key): Path<String>,
    ) -> impl IntoResponse {
        admin_panel.show_feattle(&key).await
    }

    async fn show_feattle_api_v1<F: Feattles + Sync>(
        State(admin_panel): State<Arc<AdminPanel<F>>>,
        Path(key): Path<String>,
    ) -> impl IntoResponse {
        admin_panel.show_feattle_api_v1(&key).await.map(Json)
    }

    async fn edit_feattle<F: Feattles + Sync>(
        State(admin_panel): State<Arc<AdminPanel<F>>>,
        Path(key): Path<String>,
        Form(form): Form<EditFeattleForm>,
    ) -> impl IntoResponse {
        admin_panel
            .edit_feattle(&key, &form.value_json, "admin".to_owned())
            .await
            .map(|_| Redirect::to("/"))
    }

    async fn edit_feattle_api_v1<F: Feattles + Sync>(
        State(admin_panel): State<Arc<AdminPanel<F>>>,
        Path(key): Path<String>,
        Json(request): Json<v1::EditFeattleRequest>,
    ) -> impl IntoResponse {
        admin_panel
            .edit_feattle_api_v1(&key, request)
            .await
            .map(Json)
    }

    async fn render_public_file<F: Feattles + Sync>(
        State(admin_panel): State<Arc<AdminPanel<F>>>,
        Path(file_name): Path<String>,
    ) -> impl IntoResponse {
        admin_panel.render_public_file(&file_name)
    }

    Router::new()
        .route("/", routing::get(list_feattles))
        .route("/api/v1/feattles", routing::get(list_feattles_api_v1))
        .route("/feattle/:key", routing::get(show_feattle))
        .route("/api/v1/feattle/:key", routing::get(show_feattle_api_v1))
        .route("/feattle/:key/edit", routing::post(edit_feattle))
        .route("/api/v1/feattle/:key", routing::post(edit_feattle_api_v1))
        .route("/public/:file_name", routing::get(render_public_file))
        .with_state(admin_panel)
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
