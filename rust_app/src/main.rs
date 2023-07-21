use aws_sdk_ssm::{Client, Error as OtherError};
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StorePath {
    path: String,
}

/// This is the main body for the function.
/// Write your code inside it.
/// There are some code example in the following URLs:
/// - https://github.com/awslabs/aws-lambda-rust-runtime/tree/main/examples
/// - https://github.com/aws-samples/serverless-rust-demo/
async fn function_handler(ssm_client: &Client, event: Request) -> Result<Response<Body>, Error> {
    let body = event.body();
    let body_str = std::str::from_utf8(body).expect("invalid body");

    info!(payload = %body_str,"Json Payload received");

    let path = match serde_json::from_str::<StorePath>(body_str) {
        Ok(path) => path,
        Err(err) => {
            let resp = Response::builder()
                .status(400)
                .header("content-type", "text/html")
                .body(err.to_string().into())
                .map_err(Box::new)?;
            return Ok(resp);
        }
    };

    let param = get_ssm_parameter(ssm_client, path.path).await.unwrap();

    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(param.into())
        .map_err(Box::new)?;

    // Return `Response` (it will be serialized to JSON automatically by the runtime)
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    info!("Getting SSM client");

    let config = aws_config::load_from_env().await;
    let ssm_client = Client::new(&config);

    run(service_fn(|event: Request| async {
        function_handler(&ssm_client, event).await
    }))
    .await
}

pub async fn get_ssm_parameter(ssm_client: &Client, path: String) -> Result<String, OtherError> {
    let output = ssm_client
        .get_parameter()
        .with_decryption(false)
        .name(path)
        .send()
        .await
        .expect("cannot get parameter");

    let parameter = output.parameter().unwrap().value().unwrap();

    Ok(parameter.to_string())
}
