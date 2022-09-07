pub mod hello_world {
    tonic::include_proto!("helloword");
}

use http::StatusCode;

use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};

use tracing::{info};

use opentelemetry_http::HeaderExtractor;

use tracing_opentelemetry::OpenTelemetrySpanExt;

use axum::{middleware::from_fn, Router};
use fregate::Tonicable;

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    #[tracing::instrument]
    async fn say_hello( &self, request: tonic::Request<HelloRequest>) -> Result<tonic::Response<HelloReply>, tonic::Status> {
        info!("Got a request: {:?}", request);
        
        let reply = hello_world::HelloReply {
            message: format!("Hello {}!", request.into_inner().name).into(),
        };

        Ok(tonic::Response::new(reply))
    }
}

#[tracing::instrument]
async fn middleware<B: std::fmt::Debug>(request: axum::http::Request<B>, _next: axum::middleware::Next<B>) -> Result<axum::response::Response, StatusCode>{
    let parent_cx = opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(request.headers()))
    });
       
    tracing::Span::current().set_parent(parent_cx);
    Ok(_next.run(request).await)
}

#[tokio::main]
async fn main() -> hyper::Result<()> {

    let oplt = helloword_tonic::OpentelemetryTracer::new(Some("Server service"), None);
    let _enter = oplt.m_span.enter();
    {
        let addr = &"[::1]:50051".parse().unwrap();
        let greeter = MyGreeter::default();
        let greeter_server = GreeterServer::new(greeter);

        let grpc = Router::from_tonic_service(greeter_server)
                                    .layer(from_fn(middleware));

        let service = grpc.into_make_service();
        axum::Server::bind(addr)
            .serve(service)
            .await?;
    }
    Ok(())
}
