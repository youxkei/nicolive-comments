#[derive(serde::Deserialize)]
pub struct Relive {
    #[serde(alias = "webSocketUrl")]
    pub web_socket_url: url::Url,
}

#[derive(serde::Deserialize)]
pub struct Site {
    pub relive: Relive,
}

#[derive(serde::Deserialize)]
pub struct EmbeddedData {
    pub site: Site,
}
