
#[derive(FromForm)]
pub struct GetGameRequest{
    pub difficulty: i16,
    pub color: char,
    pub username: String
}
