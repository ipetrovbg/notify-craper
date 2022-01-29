use scraper::{Html, Selector};
use serde::Deserialize;

const EXPENSIVE_PRICE: i32 = 3999;

pub struct ParseProduct {
    pub price: i32,
    pub name: String,
    pub message: String,
    html: Option<Html>,
}

impl ParseProduct {
    pub fn new(html_string: String) -> Self {
        ParseProduct {
            html: Some(Html::parse_document(&html_string)),
            name: "".to_string(),
            price: 0,
            message: "".to_string(),
        }
    }

    pub fn parse_header(self) -> ParseProduct {
        let html = self.html.unwrap();
        let header_selector = Selector::parse(".header-item").unwrap();
        let header_h1_selector = Selector::parse("h1").unwrap();
        let header_element = html.select(&header_selector).next().unwrap();
        let header_h1_element = header_element.select(&header_h1_selector).next().unwrap();
        let header_text = header_h1_element.text().collect::<Vec<_>>();
        let header = header_text.join("");

        ParseProduct {
            name: header.clone(),
            price: self.price,
            html: Some(html),
            message: header,
        }
    }

    pub fn parse_price(self) -> ParseProduct {
        let html = self.html.unwrap();

        let price_selector = Selector::parse("#our_price_display_64336").unwrap();
        let price_el = html.select(&price_selector).next().unwrap();
        let meta_selector = Selector::parse("meta").unwrap();
        let price_meta_el = price_el.select(&meta_selector).next().unwrap();
        let price_meta_value = price_meta_el.value().attrs();
        let price_meta_vec = price_meta_value.collect::<Vec<_>>();
        let mut product_price = 0;
        let mut message = self.message;
        for price in price_meta_vec.iter() {
            if price.0.contains("content") {
                let price_int = price.1.parse().unwrap_or(0);
                product_price = price_int;
                if price_int < EXPENSIVE_PRICE {
                    message = format!("Go and buy 🎉");
                } else {
                    message = format!("Still too expensive 😕");
                }
            }
        }
        ParseProduct {
            name: self.name,
            html: Some(html),
            price: product_price,
            message,
        }
    }
}

#[derive(Deserialize)]
pub struct EventBridgeScheduledEvent {
    pub time: String, // {
                      //     "version": "0",
                      //     "id": "89d1a02d-5ec7-412e-82f5-13505f849b41",
                      //     "detail-type": "Scheduled Event",
                      //     "source": "aws.events",
                      //     "account": "123456789012",
                      //     "time": "2016-12-30T18:44:49Z",
                      //     "region": "us-east-1",
                      //     "resources": ["arn:aws:events:us-east-1:123456789012:rule/SampleRule"],
                      //     "detail": {}
                      // }
}
