use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovDataResponse {
    pub response: ResponseData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseData {
    pub body: Body,
    pub header: Header,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Body {
    pub total_count: Option<i32>,
    pub items: Vec<AirItem>,
    pub page_no: Option<i32>,
    pub num_of_rows: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    pub result_msg: String,
    pub result_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AirItem {
    pub so2_grade: Option<String>,
    pub co_flag: Option<String>,
    pub khai_value: String,
    pub so2_value: String,
    pub co_value: String,
    pub pm25_flag: Option<String>,
    pub pm10_flag: Option<String>,
    pub o3_grade: Option<String>,
    pub pm10_value: String,
    pub khai_grade: Option<String>,
    pub pm25_value: String,
    pub sido_name: String,
    pub no2_flag: Option<String>,
    pub no2_grade: Option<String>,
    pub o3_flag: Option<String>,
    pub pm25_grade: Option<String>,
    pub so2_flag: Option<String>,
    pub data_time: String,
    pub co_grade: Option<String>,
    pub no2_value: String,
    pub station_name: String,
    pub pm10_grade: Option<String>,
    pub o3_value: String,
}

