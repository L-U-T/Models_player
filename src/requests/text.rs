use gloo::net::http::Request;

use super::error::RequestResult;

pub async fn request_string(path: &str) -> RequestResult<String> {
    Ok(Request::get(path)
        .header("responseType", "blob")
        .send()
        .await?
        .text()
        .await?)
}