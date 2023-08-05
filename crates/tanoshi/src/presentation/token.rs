use axum::{
    async_trait,
    extract::{FromRequestParts, TypedHeader},
    http::request::Parts,
    RequestPartsExt,
};
use headers::{authorization::Bearer, Authorization};

pub struct Token(pub String);

#[async_trait]
impl<S> FromRequestParts<S> for Token
where
    S: Send + Sync,
{
    type Rejection = ();

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Extract the token from the authorization header
        let token = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map(|TypedHeader(Authorization(bearer))| Token(bearer.token().to_string()))
            .unwrap_or_else(|_| Token("".to_string()));

        Ok(token)
    }
}
