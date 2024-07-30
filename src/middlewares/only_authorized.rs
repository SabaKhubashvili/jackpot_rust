use std::future::{ready, Ready};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http, Error, HttpMessage,
};
use futures::future::LocalBoxFuture;

use crate::jwt::decode_jwt;

pub struct OnlyAuthorized;

impl<S, B> Transform<S, ServiceRequest> for OnlyAuthorized
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = OnlyAuthorizedMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(OnlyAuthorizedMiddleware { service }))
    }
}

pub struct OnlyAuthorizedMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for OnlyAuthorizedMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let token = req.headers().get(http::header::AUTHORIZATION);
        match token {
            Some(authorization_token) => {
                let decoded = decode_jwt(authorization_token.to_str().unwrap_or(""));
                if let Ok(claims) = decoded {
                    req.extensions_mut().insert(claims);
                    let fut = self.service.call(req);
                    Box::pin(async move { fut.await })
                } else {
                    Box::pin(async {
                        Err(Error::from(actix_web::error::ErrorUnauthorized(
                            "Not Authorized!",
                        )))
                    })
                }
            }
            None => Box::pin(async {
                Err(Error::from(actix_web::error::ErrorUnauthorized(
                    "Not Authorized!",
                )))
            }),
        }
    }
}
