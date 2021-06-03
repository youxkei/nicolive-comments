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
    Room { data: RoomData },
    Ping,
    Disconnect { data: DisconnectData },
    Reconnect,

    Seat,
    Akashic,
    Stream,
    ServerTime,
    Statistics,
    Schedule,
    PostCommentResult,
    TagUpdated,
    Taxonomy,
    StreamQualities,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartWatchingData {
    pub recconect: bool,
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
pub struct DisconnectData {
    pub reason: DisconnectReason,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DisconnectReason {
    Takeover,
    NoPermission,
    EndProgram,
    PingTimemout,
    TooManyConnections,
    TooManyWatchings,
    Crowded,
    MaintenanceIn,
    ServiceTemporarilyUnavailable,
}
