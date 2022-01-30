mod model;
use crate::model::{ParseProduct, SimpleProductResponse};
use aws_lambda_events::event::cloudwatch_events::CloudWatchEvent;
use aws_sdk_sns::Client;
use dotenv::dotenv;
use lambda_runtime::{handler_fn, Context};
use log::{debug, info, LevelFilter};
use simple_logger::SimpleLogger;
use std::env;
use std::error::Error;

const URL_TO_PARSE: &str = "https://magazin.photosynthesis.bg/bg/64336-fotoaparat-sony-a7-iii.html?search_query=A7+III&results=21";

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    dotenv().ok();
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .init()
        .unwrap();

    info!("Initializing lambda");

    let handler_callback = handler_fn(handler);
    lambda_runtime::run(handler_callback).await?;

    Ok(())
}

async fn handler(
    event_bridge: CloudWatchEvent,
    _: Context,
) -> Result<SimpleProductResponse, lambda_runtime::Error> {

    info!(target: "EventBridge", "Trigger time{}", event_bridge.time);

    match make_request() {
        Ok(html_string) => {
            let shared_config = aws_config::load_from_env().await;
            let topic_arn = env::var("TOPIC_ARN")?;

            let product = ParseProduct::new(html_string).parse_header().parse_price();

            let sns_client = Client::new(&shared_config);

            let simple_product = SimpleProductResponse {
                name: product.name.clone(),
                price: product.price.clone(),
                message: product.message.clone(),
            };

            info!(target: "Product Name", "{}", simple_product.name);
            info!(target: "Product Price", "{}", simple_product.price);

            sns_client
                .publish()
                .topic_arn(topic_arn)
                .message(build_email_message(&product))
                .send()
                .await?;

            Ok(simple_product)
        }
        Err(_) => {
            log::error!(target: "Reqwest error", "Couldn't make http request");
            Err("Reqwest lib couldn't make http request".into())
        }
    }
}

fn build_email_message(product: &ParseProduct) -> String {
    format!(
        "=============== Photosynthesis ===============\n\
        Requesting information for Sony A7 IIIn\n\
        Product: {}\n\
        Price: {}\n\
        {}\n\
        Product URL: {}\n\
        \n\
        ==============================================
        ",
        product.name, product.price, product.message, URL_TO_PARSE
    )
}

fn make_request() -> Result<String, Box<dyn Error>> {
    let client = reqwest::blocking::Client::builder().build()?;
    let response = client.get(URL_TO_PARSE).send()?;
    let body_response = response.text()?;
    Ok(body_response)
}
