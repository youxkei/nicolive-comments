use chrono::{serde::ts_seconds::deserialize as deserialize_datetime, DateTime, Local};
use serde::{de::Deserializer, Deserialize, Serialize};

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum TxMessage {
    Ping(PingData),

    Thread {
        thread: String,
        version: String,
        user_id: String,
        res_from: i32,
        with_global: i32,
        scores: i32,
        nicoru: i32,
    },
}

pub fn datetime_deserializer_to_local<'a, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
where
    D: Deserializer<'a>,
{
    deserialize_datetime(deserializer).map(|deserialized| DateTime::from(deserialized))
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum RxMessage {
    Ping(PingData),

    Thread {
        #[serde(alias = "resultcode")]
        result_code: i32,
    },

    Chat {
        no: Option<i32>,
        #[serde(deserialize_with = "datetime_deserializer_to_local")]
        date: DateTime<Local>,
        mail: Option<String>,
        user_id: String,
        premium: Option<i32>,
        anonymity: Option<i32>,
        content: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "content", rename_all = "camelCase")]
pub enum PingData {
    #[serde(rename = "rf:0")]
    Rf0,
}
