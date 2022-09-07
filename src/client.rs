pub mod hello_world {
    tonic::include_proto!("helloword");
}

use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;

use tonic::{Request};

use tracing::{ info, info_span, Instrument };

#[tracing::instrument]
async fn send_hello() -> Result<(), Box<dyn std::error::Error>> {
    let channel = tonic::transport::Endpoint::from_static("http://[::1]:50051")
        .connect()
        .await
        .unwrap();
    let mut client = GreeterClient::new(channel);//::connect("http://[::1]:50051").await?;        

    let request = Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    let response = client.say_hello(helloword_tonic::inject(request)).instrument(info_span!("GreeterClient client request")).await?;

    info!("RESPONSE={:?}", response);
    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let oplt = helloword_tonic::OpentelemetryTracer::new(Some("Client_service"), None);
    let _enter = oplt.m_span.enter();
    send_hello().await?;
    
    Ok(())
}
