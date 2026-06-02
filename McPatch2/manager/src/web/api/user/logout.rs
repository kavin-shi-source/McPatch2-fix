use axum::extract::State;
use axum::response::Response;

use crate::web::api::PublicResponseBody;
use crate::web::webstate::WebState;

pub async fn api_logout(State(state): State<WebState>) -> Response {
    let mut auth = state.auth;

    auth.clear_token().await;

    if let Err(_) = auth.save().await {
        return PublicResponseBody::<()>::err("failed to save auth data");
    }

    PublicResponseBody::<()>::ok_no_data()
}