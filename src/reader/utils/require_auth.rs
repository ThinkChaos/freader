use actix_service::{Service, Transform};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{http::header, Error, HttpResponse};
use futures::future::{self, Either, FutureResult};
use futures::Poll;

use crate::AppData;

pub struct RequireAuth;

pub struct RequireAuthMiddleware<S> {
    service: S,
}

impl<S> Transform<S> for RequireAuth
where
    S: Service<
        Request = ServiceRequest,
        Response = ServiceResponse<actix_http::body::Body>,
        Error = Error,
    >,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<actix_http::body::Body>;
    type Error = Error;
    type InitError = ();
    type Transform = RequireAuthMiddleware<S>;
    type Future = FutureResult<Self::Transform, Self::InitError>;

    fn new_transform(&self, service: S) -> Self::Future {
        future::ok(RequireAuthMiddleware { service })
    }
}

impl<S> Service for RequireAuthMiddleware<S>
where
    S: Service<
        Request = ServiceRequest,
        Response = ServiceResponse<actix_http::body::Body>,
        Error = Error,
    >,
    S::Future: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<actix_http::body::Body>;
    type Error = Error;
    type Future = Either<S::Future, FutureResult<Self::Response, Self::Error>>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.service.poll_ready()
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let app_data = req.app_data::<AppData>().unwrap();

        let authorized = req
            .headers()
            .get(header::AUTHORIZATION)
            .map(|val| val == &format!("GoogleLogin auth={}", app_data.cfg.auth_password))
            .unwrap_or(false);

        if authorized {
            Either::A(self.service.call(req))
        } else {
            let json_content_type = req
                .headers()
                .get(header::CONTENT_TYPE)
                .map(|val| val == "application/json")
                .unwrap_or(false);

            let json_output_query = req.query_string().contains("output=json");

            let response = if json_content_type || json_output_query {
                HttpResponse::Unauthorized()
                    .content_type("application/json")
                    .body(r#"{"error":"Unauthorized"}"#)
            } else {
                HttpResponse::Unauthorized().body("Unauthorized")
            };

            Either::B(future::ok(req.into_response(response)))
        }
    }
}
