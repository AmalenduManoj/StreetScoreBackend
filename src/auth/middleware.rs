use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, http::Method, HttpResponse,
};
use futures::future::LocalBoxFuture;
use crate::auth::jwt::verify_token;

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        std::future::ready(Ok(AuthMiddlewareService { service }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
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
        // Allow CORS preflight requests to proceed so CORS middleware can respond
        if req.method() == &Method::OPTIONS {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await });
        }

        let token = req.headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .map(|s| s.to_string());

        match token {
            Some(token) => {
                match verify_token(&token) {
                    Ok(claims) => {
                        req.extensions_mut().insert(claims);
                        let fut = self.service.call(req);
                        Box::pin(async move {
                            fut.await
                        })
                    }
                    Err(_) => {
                        Box::pin(async move {
                            let origin = req
                                .headers()
                                .get("Origin")
                                .and_then(|v| v.to_str().ok())
                                .unwrap_or("*");

                            let resp = HttpResponse::Unauthorized()
                                .insert_header(("Access-Control-Allow-Origin", origin))
                                .insert_header(("Access-Control-Allow-Credentials", "true"))
                                .body("Invalid token");

                            let internal = actix_web::error::InternalError::from_response("", resp);
                            Err(internal.into())
                        })
                    }
                }
            }
            None => {
                Box::pin(async move {
                    let origin = req
                        .headers()
                        .get("Origin")
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("*");

                    let resp = HttpResponse::Unauthorized()
                        .insert_header(("Access-Control-Allow-Origin", origin))
                        .insert_header(("Access-Control-Allow-Credentials", "true"))
                        .body("Missing Authorization header");

                    let internal = actix_web::error::InternalError::from_response("", resp);
                    Err(internal.into())
                })
            }
        }
    }
}