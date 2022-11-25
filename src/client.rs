pub mod hello_world {
    tonic::include_proto!("helloword");
}

use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;

use tonic::Request;

use tracing::{info, info_span, Instrument};

use metrics::{
    decrement_gauge, describe_counter, describe_histogram, gauge, histogram, increment_counter,
    increment_gauge, register_histogram,
};

use metrics_exporter_prometheus::PrometheusBuilder;
use metrics_util::MetricKindMask;

use std::thread;
use std::time::Duration;

#[tracing::instrument]
async fn send_hello() -> Result<(), Box<dyn std::error::Error>> {
    let channel = tonic::transport::Endpoint::from_static("http://0.0.0.0:3000")
        .connect()
        .await
        .unwrap();
    let mut client = GreeterClient::new(channel); //::connect("http://[::1]:50051").await?;

    let request = Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    let response = client
        .say_hello(helloword_tonic::inject(request))
        .instrument(info_span!("GreeterClient client request"))
        .await?;

    info!("RESPONSE={:?}", response);
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let oplt = helloword_tonic::OpentelemetryTracer::new(Some("Client_service"), None);
    let _enter = oplt.m_span.enter();

    // let builder = PrometheusBuilder::new();
    // builder
    //     .with_push_gateway(
    //         "http://0.0.0.0:9000/metrics/job/example",
    //         Duration::from_secs(1),
    //     )
    //     .expect("push gateway endpoint should be valid")
    //     .idle_timeout(
    //         MetricKindMask::COUNTER | MetricKindMask::HISTOGRAM,
    //         Some(Duration::from_secs(1)),
    //     )
    //     .install()
    //     .expect("failed to install Prometheus recorder");

    // describe_histogram!(
    //     "send_hello_delta_sec",
    //     "Time for send request and get redponse."
    // );

    // let clock = quanta::Clock::new();
    //let current = clock.now();

    send_hello().await?;

    //histogram!("send_hello_delta_sec", clock.now() - current, "system" => "foo");
    Ok(())
}
