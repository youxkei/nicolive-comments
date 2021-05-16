mod embedded_data;
mod message_server;
mod relive;

use embedded_data::EmbeddedData;
use futures_util::{SinkExt, StreamExt};
use scraper::{Html, Selector};
use std::cell::RefCell;
use std::rc::Rc;
use tokio::{join, sync::mpsc::channel};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message::Text};
use url::Url;

/// A command-line tool for fetching comments from nicolive.
#[derive(structopt::StructOpt, Debug)]
struct Args {
    /// A live URL whose comments will be fatched
    #[structopt(name = "URL", parse(try_from_str))]
    url: Url,

    /// Outputs comments with JSON format.
    #[structopt(short, long)]
    json: bool,
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

    let (message_server_tx, mut message_server_rx) = channel(1);

    let pong_message_serialized = serde_json::to_string(&relive::TxMessage::Pong)?;
    let keep_seat_message_serialized = serde_json::to_string(&relive::TxMessage::KeepSeat)?;

    let start_watching_message = relive::TxMessage::StartWatching {
        data: relive::StartWatchingData {
            room: relive::StartWatchingDataRoom {
                protocol: "webSocket".to_string(),
                commentable: false,
            },
            recconect: false,
        },
    };

    relive_writer
        .send(Text(serde_json::to_string(&start_watching_message)?))
        .await?;

    let relive_writer = Rc::new(RefCell::new(relive_writer));

    let receive_relive_message = relive_reader.for_each(|message| async {
        let message: relive::RxMessage =
            serde_json::from_slice(&message.unwrap().into_data()).unwrap();

        match message {
            relive::RxMessage::Ping => {
                let mut relive_writer = relive_writer.borrow_mut();

                relive_writer
                    .send(Text(pong_message_serialized.clone()))
                    .await
                    .unwrap();

                relive_writer
                    .send(Text(keep_seat_message_serialized.clone()))
                    .await
                    .unwrap();
            }

            relive::RxMessage::Room { data } => message_server_tx
                .send((data.message_server.uri, data.thread_id))
                .await
                .unwrap(),

            _ => {}
        }
    });

    let receive_message = async move {
        let (message_server_url, thread) = message_server_rx.recv().await.unwrap();
        let (message_stream, _response) = connect_async(message_server_url).await.unwrap();

        let (mut message_writer, message_reader) = message_stream.split();

        let thread_message = message_server::TxMessage::Thread {
            thread: thread.to_string(),
            version: "20061206".to_string(),
            user_id: "guest".to_string(),
            res_from: -150,
            with_global: 1,
            scores: 1,
            nicoru: 0,
        };

        message_writer
            .send(Text(serde_json::to_string(&thread_message).unwrap()))
            .await
            .unwrap();

        message_reader
            .for_each(|message| async {
                let message: message_server::RxMessage =
                    serde_json::from_slice(&message.unwrap().into_data()).unwrap();

                match message {
                    message_server::RxMessage::Chat { ref content, .. } => {
                        if args.json {
                            println!("{}", serde_json::to_string(&message).unwrap());
                        } else {
                            println!("{}", content)
                        }
                    }

                    _ => {}
                }
            })
            .await;
    };

    join!(receive_relive_message, receive_message);

    Ok(())
}
