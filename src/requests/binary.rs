use gloo::net::http::Request;

use super::error::RequestResult;

pub async fn request_binary(path: &str) -> RequestResult<Vec<u8>> {
    Ok(Request::get(path)
        .header("responseType", "blob")
        .send()
        .await?
        .binary()
        .await?)
}