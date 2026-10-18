use serde::Deserialize;


#[derive(Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ResponseType {
    JSON,
    TEXT,
    FILE,
    HTMLFILE,
    TEMPLATE,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteResponse {
    pub response_code: u16,
    pub data: String,
    pub response_type: ResponseType,
}