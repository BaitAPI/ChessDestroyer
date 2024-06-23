#[derive(FromForm)]
pub struct GameSettings {
    pub new_session: Option<bool>,
    pub username: String,
    pub difficulty: i16,
    pub color: char,
}

