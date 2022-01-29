mod model;
use aws_sdk_sns::Client;
use dotenv::dotenv;
use lambda_runtime::Context;
use log::{debug, error, info};
use std::env;
use std::error::Error;

const URL_TO_PARSE: &str = "https://magazin.photosynthesis.bg/bg/64336-fotoaparat-sony-a7-iii.html?search_query=A7+III&results=21";

#[tokio::main]
async fn main() -> Result<(), lambda_runtime::Error> {
    dotenv().ok();
    debug!("Initializing lambda.");
    let handler = lambda_runtime::handler_fn(handler);
    lambda_runtime::run(handler).await?;
    Ok(())
}

async fn handler(_: String, _: Context) -> Result<bool, lambda_runtime::Error> {
    match make_request() {
        Ok(html_string) => {
            info!("{}", env::var("TOPIC_ARN")?);
            let product = model::ParseProduct::new(html_string)
                .parse_header()
                .parse_price();

            let shared_config = aws_config::load_from_env().await;
            let client = Client::new(&shared_config);
            let topic_arn = env::var("TOPIC_ARN")?;

            client
                .publish()
                .topic_arn(topic_arn)
                .message(build_email_message(product))
                .send()
                .await?;
        }
        Err(_) => {
            error!("Couldn't make http request");
        }
    };
    Ok(true)
}

fn build_email_message(product: model::ParseProduct) -> String {
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
