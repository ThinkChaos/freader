use actix_service::{Service, Transform};
use actix_web::{Error, HttpResponse};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use futures::Poll;
use futures::future::{self, Either, FutureResult};


pub struct RequireAuth;

pub struct RequireAuthMiddleware<S> {
    service: S,
}

impl<S, B> Transform<S> for RequireAuth
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequireAuthMiddleware<S>;
    type Future = FutureResult<Self::Transform, Self::InitError>;

    fn new_transform(&self, service: S) -> Self::Future {
        future::ok(RequireAuthMiddleware { service })
    }
}

impl<S, B> Service for RequireAuthMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Either<S::Future, FutureResult<Self::Response, Self::Error>>;

    fn poll_ready(&mut self) -> Poll<(), Self::Error> {
        self.service.poll_ready()
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let app_data = req.app_data::<crate::Data>().unwrap();

        let authorized = req.headers().get("Authorization")
            .map(|val| val == &format!("GoogleLogin auth={}", app_data.secret))
            .unwrap_or(false);

        if authorized {
            Either::A(self.service.call(req))
        }
        else {
            Either::B(future::ok(req.into_response(
                HttpResponse::Unauthorized().finish().into_body()
            )))
        }
    }
}
