mod embedded_data;
mod message_server;
mod relive;

use embedded_data::EmbeddedData;
use futures_util::{SinkExt, StreamExt};
use http::Request;
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

    /// Fetches about <num-comments> comments.
    /// Must be in the range 0-200
    #[structopt(short, long, default_value = "150")]
    num_comments: i32,

    /// Outputs comments with JSON format.
    #[structopt(short, long)]
    json: bool,

    /// Outputs appended comments
    #[structopt(short, long)]
    follow: bool,
}

#[paw::main]
#[tokio::main]
pub async fn main(args: Args) -> Result<(), Box<dyn std::error::Error>> {
    if args.num_comments < 0 || 200 < args.num_comments {
        eprintln!(concat!(
            "\x1B[31m",
            "error:",
            "\x1B[0m",
            "Invalid value for '",
            "\x1B[33m",
            "--num-comments <num-comments>",
            "\x1B[0m",
            "': must be in the range 0-200"
        ));
        std::process::exit(1);
    }

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

    let web_socket_request = Request::builder()
        .uri(embedded_data.site.relive.web_socket_url.to_string())
        .header("User-Agent", "https://github.com/youxkei/nicolive-comments")
        .body(())
        .unwrap();

    let (relive_stream, _response) = connect_async(web_socket_request).await?;

    let (mut relive_writer, relive_reader) = relive_stream.split();

    let (message_server_tx, mut message_server_rx) = channel(1);

    let pong_message_serialized = serde_json::to_string(&relive::TxMessage::Pong)?;
    let keep_seat_message_serialized = serde_json::to_string(&relive::TxMessage::KeepSeat)?;

    let start_watching_message = relive::TxMessage::StartWatching {
        data: relive::StartWatchingData { recconect: false },
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

                // TODO: use keep_interval_sec of seat message
                relive_writer
                    .send(Text(keep_seat_message_serialized.clone()))
                    .await
                    .unwrap();
            }

            relive::RxMessage::Room { data } => message_server_tx
                .send((data.message_server.uri, data.thread_id))
                .await
                .unwrap(),

            relive::RxMessage::Disconnect { data } => {
                eprintln!("Disconnected from server. reason: {:?}", data.reason);
                std::process::exit(0);
            }

            _ => {}
        }
    });

    let receive_message = async move {
        let (message_server_url, thread) = message_server_rx.recv().await.unwrap();
        let (message_stream, _response) = connect_async(message_server_url).await.unwrap();

        let (mut message_writer, message_reader) = message_stream.split();

        let thread_messages = vec![
            message_server::TxMessage::Thread {
                thread: thread.to_string(),
                version: "20061206".to_string(),
                user_id: "guest".to_string(),
                res_from: -args.num_comments,
                with_global: 1,
                scores: 1,
                nicoru: 0,
            },
            message_server::TxMessage::Ping(message_server::PingData::Rf0),
        ];

        message_writer
            .send(Text(serde_json::to_string(&thread_messages).unwrap()))
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

                    message_server::RxMessage::Ping(message_server::PingData::Rf0) => {
                        if !args.follow {
                            std::process::exit(0);
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
