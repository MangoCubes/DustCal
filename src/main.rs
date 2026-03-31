use axum::extract::State;
use axum::response::{Html, IntoResponse};
use axum::{Router, extract::Query, http::StatusCode, routing::get};
use chrono::Utc;
use dotenvy::dotenv;
use moka::future::Cache;
use serde::Deserialize;
use std::env;
use std::time::Duration;

const VALID_REGIONS: &[&str] = &[
    "서울", "부산", "대구", "인천", "광주", "대전", "울산", "경기", "강원", "충북", "충남", "전북", "전남", "경북", "경남", "제주", "세종",
   
];

use once_cell::sync::Lazy;

pub static HTML: Lazy<String> = Lazy::new(|| {
    let mut html =
        String::from("<!DOCTYPE html><html><head><title>시/도를 선택해주세요</title></head><body>");
    html.push_str("<h1>시/도를 선택해주세요</h1><ul>");
    for region in VALID_REGIONS {
        html.push_str(&format!(
            "<li><a href=\"/location?region={}\">{}</a></li>",
            region, region
        ));
    }
    html.push_str("</ul></body></html>");
    html
});

pub mod govdata;

use crate::govdata::GovDataResponse;

#[derive(Deserialize)]
struct LocationParams {
    region: Option<String>,
    station: Option<String>,
}

#[derive(Clone)]
struct AppState {
    cache: Cache<String, GovDataResponse>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let cache = Cache::builder()
        .time_to_live(Duration::from_secs(3600))
        .build();
    let state = AppState { cache };

    let app = Router::new()
        .route("/location", get(use_location))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    println!(
        "Starting DustCal on port {}!",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.unwrap();
}

async fn use_location(
    State(state): State<AppState>,
    Query(params): Query<LocationParams>,
) -> impl axum::response::IntoResponse {
    let Some(region) = params.region else {
        return Html(HTML.as_str()).into_response();
    };
    if !VALID_REGIONS.contains(&&region.as_str()) {
        return Html(HTML.as_str()).into_response();
    };
    let api_key = env::var("GOVDATA_API_KEY").unwrap();

    let data = if let Some(cached_result) = state.cache.get(&region).await {
        cached_result
    } else {
        let client = reqwest::Client::new();
        let url = format!(
            "https://apis.data.go.kr/B552584/ArpltnInforInqireSvc/getCtprvnRltmMesureDnsty?serviceKey={}&returnType=json&numOfRows=100&pageNo=1&sidoName={}&ver=1.0",
            api_key, region
        );

        let response = client.get(url).send().await;

        match response {
            Ok(res) => {
                if res.status().is_success() {
                    match res.json::<GovDataResponse>().await {
                        Ok(data) => {
                            if data.response.header.result_code != "00" {
                                return (
                                    StatusCode::BAD_GATEWAY,
                                    format!(
                                        "API 문제가 발생했습니다! contact@skew.ch <- 여기로 메일 보내주세요. 에러: {}",
                                        data.response.header.result_msg
                                    ),
                                )
                                    .into_response();
                            }
                            if data.response.body.items.is_empty() {
                                return (
                                    StatusCode::NOT_FOUND,
                                    format!("{}의 데이터가 존재하지 않습니다.", region),
                                )
                                    .into_response();
                            }
                            state.cache.insert(region.clone(), data.clone()).await;
                            data
                        }
                        Err(e) => {
                            return (
                                StatusCode::INTERNAL_SERVER_ERROR,
                                format!("데이터 파싱 실패했습니다! contact@skew.ch <- 여기로 메일 보내주세요. 에러: {}", e),
                            )
                                .into_response();
                        }
                    }
                } else {
                    return (
                        StatusCode::BAD_GATEWAY,
                        format!("공공데이터포털 에러: {}", res.status()),
                    )
                        .into_response();
                }
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("요청 실패: {}", e),
                )
                    .into_response();
            }
        }
    };

    if let Some(station_name) = &params.station {
        if let Some(item) = data
            .response
            .body
            .items
            .iter()
            .find(|i| &i.station_name == station_name)
        {
            return (
                [(axum::http::header::CONTENT_TYPE, "text/calendar")],
                generate_ics(item),
            )
                .into_response();
        }
    }

    let mut html = format!(
        "<!DOCTYPE html><html><head><title>{} 내 구역</title></head><body><h1>{} 내 구역</h1><ul>",
        region, region
    );
    for item in &data.response.body.items {
        html.push_str(&format!(
            "<li><a href=\"/location?region={}&station={}\">{}</a></li>",
            region, item.station_name, item.station_name
        ));
    }
    html.push_str("</ul></body></html>");
    Html(html).into_response()
}

fn grade_to_emoji(grade: &Option<String>) -> &str {
    if let Some(g) = grade {
        match g.as_str() {
            "1" => "🟢",
            "2" => "🟡",
            "3" => "🟠",
            "4" => "🔴",
            _ => "?",
        }
    } else {
        "?"
    }
}

fn generate_ics(item: &crate::govdata::AirItem) -> String {
    let now = Utc::now();
    let today = now.date_naive();
    let tomorrow = today.succ_opt().unwrap_or(today);

    let dtstart = today.format("%Y%m%d").to_string();
    let dtend = tomorrow.format("%Y%m%d").to_string();

    let summary = format!(
        "PM2.5: {} ({}), PM10: {} ({}) ({})",
        item.pm25_value,
        grade_to_emoji(&item.pm25_grade),
        item.pm10_value,
        grade_to_emoji(&item.pm10_grade),
        item.station_name
    );

    let description = format!(
        "PM2.5: {}\\nPM10: {}\\n시/도명: {}\\n구역: {}\\n마지막 갱신: {}",
        item.pm25_value, item.pm10_value, item.sido_name, item.station_name, item.data_time
    );

    let uid = format!("dustcal_{}", item.station_name.replace(" ", "_"));

    format!(
        "BEGIN:VCALENDAR\r\n\
         VERSION:2.0\r\n\
         PRODID://DustCal//EN\r\n\
         BEGIN:VEVENT\r\n\
         UID:{}\r\n\
         DTSTART;VALUE=DATE:{}\r\n\
         DTEND;VALUE=DATE:{}\r\n\
         SUMMARY:{}\r\n\
         DESCRIPTION:{}\r\n\
         END:VEVENT\r\n\
         END:VCALENDAR\r\n",
        uid, dtstart, dtend, summary, description
    )
}
