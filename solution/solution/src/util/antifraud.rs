use chrono::Utc;
use reqwest::{Client, Response, StatusCode};
use uuid::Uuid;

use crate::{
    database::redis::RedisPool,
    models::{AntiFraudRequest, AntiFraudResponse},
    routes::ApiError,
    ANTIFRAUD_ADDRESS,
};

const ANTIFRAUD_NAMESPACE: &str = "antifraud";

pub async fn ask(email: String, promo_id: Uuid, cache: &RedisPool) -> Result<bool, ApiError> {
    let mut cache = cache.connect().await?;

    if let Some(ok) = cache.get(ANTIFRAUD_NAMESPACE, &email).await? {
        return Ok(ok.parse::<bool>().unwrap_or(false));
    }

    let client = Client::new();
    let request = AntiFraudRequest {
        user_email: email.clone(),
        promo_id: promo_id.to_string(),
    };

    let mut res: Option<Response> = None;
    for _ in 0..3 {
        res = if let Ok(res) = client
            .post(format!("http://{}/api/validate", ANTIFRAUD_ADDRESS()))
            .json(&request)
            .send()
            .await
        {
            Some(res)
        } else {
            None
        };

        if let Some(resp) = &res {
            if resp.status() == StatusCode::OK {
                break;
            } else {
                res = None;
            }
        }
    }

    let res = if let Some(res) = res {
        res
    } else {
        return Ok(false);
    };

    let res: AntiFraudResponse = res.json().await?;
    if let Some(cache_until) = res.cache_until {
        let cache_duration = (cache_until - Utc::now()).num_milliseconds();
        cache
            .set(
                ANTIFRAUD_NAMESPACE,
                &email,
                &res.ok.to_string(),
                Some(cache_duration),
            )
            .await?;
    };

    Ok(res.ok)
}
