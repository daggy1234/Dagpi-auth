use actix_service::{Service, Transform};
use actix_web::{dev::ServiceRequest, dev::ServiceResponse, Error, HttpResponse};
use futures::future::{ok, Ready};
use futures::Future;
use serde::Serialize;
use std::cell::RefCell;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

#[derive(Serialize)]
struct ErrorResp<'a> {
    message: &'a str,
}

pub struct RequiresAuth;

// Middleware factory is `Transform` trait from actix-service crate
// `S` - type of the next service
// `B` - type of response's body
impl<S: 'static, B> Transform<S> for RequiresAuth
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequiresAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RequiresAuthMiddleware {
            service: Rc::new(RefCell::new(service)),
        })
    }
}

pub struct RequiresAuthMiddleware<S> {
    service: Rc<RefCell<S>>,
}

impl<S, B> Service for RequiresAuthMiddleware<S>
where
    S: Service<Request = ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Request = ServiceRequest;
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&mut self, req: ServiceRequest) -> Self::Future {
        let mut svc = self.service.clone();

        Box::pin(async move {
            let header = req.headers().get("Authorization");
            let authorised: bool = match header {
                Some(header_val) => match header_val.to_str() {
                    Ok(val) => {
                        let tok = match std::env::var("TOKEN") {
                            Ok(val) => val,
                            Err(error) => panic!("{}", error),
                        };
                        val == tok
                    }
                    Err(error) => {
                        print!("{}", error);
                        false
                    }
                },
                None => {
                    print!("NONE");
                    false
                }
            };
            print!("{}", authorised);
            if authorised {
                Ok(svc.call(req).await.unwrap())
            } else {
                Ok(req.into_response(
                    HttpResponse::Forbidden()
                        .json(ErrorResp {
                            message: "You are not authorised",
                        })
                        .into_body(),
                ))
            }
        })
    }
}
