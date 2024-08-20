use actix_service::{Service, Transform};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error};
use futures::future::{ok, Ready, LocalBoxFuture};
use std::future::{ready, Ready as OtherReady};
use std::task::{Context, Poll};

pub struct Logging;

impl<S, B> Transform<S, ServiceRequest> for Logging
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = LoggingMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(LoggingMiddleware { service })
    }
}

pub struct LoggingMiddleware<S> {
    service: S,
}

// Implement the Service trait for your middleware
impl<S, B> Service<ServiceRequest> for LoggingMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        println!("Request: {:?}", req); // Log the incoming request

        let fut = self.service.call(req);

        Box::pin(async move {
            let res = fut.await?;
            println!("Response Status: {:?}", res.status());
            println!("Response Headers: {:?}", res.headers());

            Ok(res)
        })
    }
}
