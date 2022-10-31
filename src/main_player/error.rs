use thiserror::Error;

pub type PlayerErrorResult<T> = Result<T, MainPlayerError>;

#[derive(Error, Debug)]
pub enum MainPlayerError {
    #[error("Cannt get error from your browser.")]
    RequestDeviceError(#[from] wgpu::RequestDeviceError),
    #[error("Use wgpu state without init.")]
    StateNotInitError,
    #[error("Something wrong with the surface of wgpu state.")]
    SurfaceError(#[from] wgpu::SurfaceError),
    #[error("Failed to load resources from static.")]
    LoadError(#[from] tobj::LoadError),
    #[error("Fail to have a net request.")]
    RequestError(#[from] crate::requests::RequestError)
}
