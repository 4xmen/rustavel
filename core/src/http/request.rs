use axum::{
    body::Body,
    extract::FromRequest,
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::de::DeserializeOwned;
use macros_core::{CheckMateValidator};

pub struct ValidatedJson<T>(pub T);

impl<S, T> FromRequest<S, Body> for ValidatedJson<T>
where
    S: Send + Sync,
    T: DeserializeOwned + CheckMateValidator,
{
    type Rejection = Response;

    async fn from_request(
        req: Request<Body>,
        state: &S,
    ) -> Result<Self, Self::Rejection> {

        let Json(payload) =
            Json::<T>::from_request(req, state)
                .await
                .map_err(IntoResponse::into_response)?;

        if let Err(errors) = payload.validate() {
            let mut response = Json(errors).into_response();
            *response.status_mut() = StatusCode::UNPROCESSABLE_ENTITY;
            return Err(response);
        }

        Ok(ValidatedJson(payload))
    }
}
