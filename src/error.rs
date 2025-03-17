use crate::fileo::error::FileoError;
use crate::member::error::MembershipError;
use crate::uda::error::UdaError;
use crate::web::error::WebError;
use thiserror::Error;

pub type Result<T, E = ApplicationError> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("An error has been encountered while executing requests onto another server.")]
    Web(#[from] WebError),
    #[error("Error while working with Fileo.")]
    Fileo(#[from] FileoError),
    #[error("Error while working with UDA.")]
    Uda(#[from] UdaError),
    #[error("Error while working with memberships.")]
    Membership(#[from] MembershipError),
}
