use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Relive {
    #[serde(alias = "webSocketUrl")]
    pub web_socket_url: url::Url,
}

#[derive(Deserialize, Debug)]
pub struct Site {
    pub relive: Relive,
}

#[derive(Deserialize, Debug)]
pub struct EmbeddedData {
    pub site: Site,
}
