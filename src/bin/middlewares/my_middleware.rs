use actix_session::SessionExt;
use actix_web::{
    Error, HttpMessage, HttpResponse,
    body::EitherBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures_util::future::LocalBoxFuture;
use futures_util::{FutureExt, TryFutureExt};
use log::info;
use mail::models::UserSession;
use std::future::{Ready, ready};

pub struct RateLimit;

impl<S, B> Transform<S, ServiceRequest> for RateLimit
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = UserSessionService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(UserSessionService { service }))
    }
}

pub struct UserSessionService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for UserSessionService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let session = req.get_session();

        if let Ok(Some(user_session)) = session.get::<UserSession>("user") {
            req.extensions_mut().insert(user_session);

            self.service
                .call(req)
                .map_ok(ServiceResponse::map_into_left_body)
                .boxed_local()
        } else {
            Box::pin(async {
                Ok(req.into_response(HttpResponse::Unauthorized().finish().map_into_right_body()))
            })
        }
    }
}
