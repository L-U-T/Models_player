use thiserror::Error;

pub type RequestResult<T> = Result<T, RequestError>;

#[derive(Error, Debug)]
pub enum RequestError {
    #[error("Fail to have a net request.")]
    NetRequestError(#[from] gloo::net::Error),
}
