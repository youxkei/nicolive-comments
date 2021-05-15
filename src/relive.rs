use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Message {
    ServerTime { data: ServerTimeData },
    Seat,
    Stream,
    Room { data: RoomData },
    Statistics { data: StatisticsData },
    Ping,
    Pong,
    KeepSeat,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ServerTimeData {
    pub current_ms: DateTime<Local>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RoomData {
    pub message_server: MessageServer,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageServer {
    pub uri: url::Url,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StatisticsData {
    pub viewers: i32,
    pub comments: i32,
    pub ad_points: i32,
    pub gift_points: i32,
}
