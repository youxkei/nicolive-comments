use chrono::{serde::ts_seconds::deserialize as deserialize_datetime, DateTime, Local};
use serde::{de::Deserializer, Deserialize, Serialize};

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub enum TxMessage {
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
    Thread {
        #[serde(alias = "resultcode")]
        result_code: i32,
    },

    Chat {
        #[serde(deserialize_with = "datetime_deserializer_to_local")]
        date: DateTime<Local>,
        user_id: String,
        content: String,
    },
}
