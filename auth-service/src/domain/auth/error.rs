#[derive(Debug)]
pub enum DomainError {
    NotFound,
    Conflict,
    InvalidCredentials,
    InactiveAccount,
    RepositoryFailure,
}
