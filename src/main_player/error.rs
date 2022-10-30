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
}
