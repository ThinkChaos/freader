use actix_service::{Service, Transform};
use actix_web::{http::header, HttpResponse};
use futures::future::{self, Either, Ready};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::utils::HttpService;
use crate::AppData;

pub struct RequireAuth;

pub struct RequireAuthMiddleware<S> {
    service: S,
}

impl<S> Transform<S> for RequireAuth
where
    S: HttpService,
    S::Future: 'static,
{
    type Request = <S as Service>::Request;
    type Response = <S as Service>::Response;
    type Error = <S as Service>::Error;
    type InitError = ();
    type Transform = RequireAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        future::ok(RequireAuthMiddleware { service })
    }
}

impl<S> Service for RequireAuthMiddleware<S>
where
    S: HttpService,
    S::Future: 'static + Sized,
{
    type Request = <S as Service>::Request;
    type Response = <S as Service>::Response;
    type Error = <S as Service>::Error;
    type Future =
        Either<S::Future, Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>>;

    fn poll_ready(&mut self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&mut self, req: Self::Request) -> Self::Future {
        let app_data = req
            .app_data::<AppData>()
            .expect("Could not extract AppData");

        let authorized = req
            .headers()
            .get(header::AUTHORIZATION)
            .map(|val| val == &format!("GoogleLogin auth={}", app_data.cfg.auth_password))
            .unwrap_or(false);

        if authorized {
            return Either::Left(self.service.call(req));
        }

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

        Either::Right(Box::pin(future::ok(req.into_response(response))))
    }
}
