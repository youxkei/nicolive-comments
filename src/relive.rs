use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum TxMessage {
    StartWatching { data: StartWatchingData },
    Pong,
    KeepSeat,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum RxMessage {
    ServerTime { data: ServerTimeData },
    Seat,
    Room { data: RoomData },
    Statistics { data: StatisticsData },
    Ping,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartWatchingData {
    pub room: StartWatchingDataRoom,
    pub recconect: bool,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartWatchingDataRoom {
    pub protocol: String,
    pub commentable: bool,
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
    pub thread_id: String,
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
