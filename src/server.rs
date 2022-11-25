pub mod hello_world {
    tonic::include_proto!("helloword");
}
use std::time::Duration;

use http::StatusCode;

use tracing::info;

use opentelemetry_http::HeaderExtractor;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use axum::{middleware::from_fn, Router};
use fregate::Tonicable;

use metrics::{
    decrement_gauge, describe_counter, describe_histogram, gauge, histogram, increment_counter,
    increment_gauge,
};

use metrics_exporter_prometheus::PrometheusBuilder;
use metrics_util::MetricKindMask;

use hello_world::greeter_server::{Greeter, GreeterServer};
use hello_world::{HelloReply, HelloRequest};

#[derive(Debug, Default)]
pub struct MyGreeter {}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    #[tracing::instrument]
    async fn say_hello(
        &self,
        request: tonic::Request<HelloRequest>,
    ) -> Result<tonic::Response<HelloReply>, tonic::Status> {
        info!("Got a request: {:?}", request);

        let reply = hello_world::HelloReply {
            message: format!("Hello {}!", request.into_inner().name).into(),
        };

        increment_counter!("request_counter", "system" => "foo");

        Ok(tonic::Response::new(reply))
    }
}

#[tracing::instrument]
async fn middleware<B: std::fmt::Debug>(
    request: axum::http::Request<B>,
    _next: axum::middleware::Next<B>,
) -> Result<axum::response::Response, StatusCode> {
    let parent_cx = opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&HeaderExtractor(request.headers()))
    });
    increment_gauge!("middleware_call", 1.0);

    tracing::Span::current().set_parent(parent_cx);
    Ok(_next.run(request).await)
}

#[tokio::main]
async fn main() -> hyper::Result<()> {
    let oplt = helloword_tonic::OpentelemetryTracer::new(Some("Server service"), None);

    PrometheusBuilder::new()
        .idle_timeout(
            MetricKindMask::COUNTER | MetricKindMask::HISTOGRAM,
            Some(Duration::from_secs(10)),
        )
        .install()
        .expect("failed to install Prometheus recorder");
    describe_counter!("request_counter", "count of incomming request");

    increment_counter!("idle_metric");
    gauge!("My_Super_testing", 42.0);

    let _enter = oplt.m_span.enter();
    {
        //let addr = &"[::1]:50051".parse().unwrap();
        let addr = &"0.0.0.0:3000".parse().unwrap();

        let greeter = MyGreeter::default();
        let greeter_server = GreeterServer::new(greeter);

        let grpc = Router::from_tonic_service(greeter_server).layer(from_fn(middleware));

        let service = grpc.into_make_service();
        axum::Server::bind(addr).serve(service).await?;
    }
    Ok(())
}
