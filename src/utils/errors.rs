pub mod internal {
    pub type InternalResult<T> = Result<T, &'static str>;
}

pub mod external {
    use rocket::http::Status;

    pub type Response<T> = Result<T, (Status, &'static str)>;
    pub type FenResponse = Result<String, (Status, String)>;
    pub type OkOrResponse<T> = Result<Status, (Status, T)>;
}