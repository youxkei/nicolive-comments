use embedded_data::EmbeddedData;
use futures_util::{SinkExt, StreamExt};
use scraper::{Html, Selector};
use std::cell::RefCell;
use std::rc::Rc;
use tokio::io::AsyncWriteExt;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

mod embedded_data;

/// A command-line tool for fetching comments from nicolive.
#[derive(structopt::StructOpt, Debug)]
struct Args {
    /// A live URL whose comments will be fatched
    #[structopt(name = "URL", parse(try_from_str))]
    url: Url,
}

#[paw::main]
#[tokio::main]
pub async fn main(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    let embedded_data_selector = Selector::parse("#embedded-data").unwrap();

    let live_page_html = Html::parse_document(
        &reqwest::get(args.url.clone())
            .await?
            .error_for_status()?
            .text()
            .await?,
    );

    let embedded_data: EmbeddedData = serde_json::from_str(
        live_page_html
            .select(&embedded_data_selector)
            .next()
            .ok_or(format!("'#embedded-data' not found in '{}'", &args.url))?
            .value()
            .attr("data-props")
            .ok_or(format!(
                "'#embedded-data' in '{}' do not have 'data-props' attribute",
                &args.url
            ))?,
    )?;

    let (relive_stream, _response) =
        connect_async(embedded_data.site.relive.web_socket_url).await?;

    let (mut relive_writer, relive_reader) = relive_stream.split();

    relive_writer.send(Message::Text("{\"type\":\"startWatching\",\"data\":{\"stream\":{\"quality\":\"abr\",\"protocol\":\"hls\",\"latency\":\"low\",\"chasePlay\":false},\"room\":{\"protocol\":\"webSocket\",\"commentable\":true},\"reconnect\":false}}".to_string())).await?;

    let relive_writer = Rc::new(RefCell::new(relive_writer));

    relive_reader
        .for_each(|message| async {
            let message_data = message.unwrap().into_data();
            let message = std::str::from_utf8(&message_data).unwrap();
            println!("{}", message);

            if message == r#"{"type":"ping"}"# {
                relive_writer
                    .borrow_mut()
                    .send(Message::Text(r#"{"type":"pong"}"#.to_string()))
                    .await
                    .unwrap();

                relive_writer
                    .borrow_mut()
                    .send(Message::Text(r#"{"type":"keepSeat"}"#.to_string()))
                    .await
                    .unwrap();
            }
        })
        .await;

    Ok(())
}
