// Importing the `FromForm` trait from the `rocket` crate.
use rocket::form::FromForm;

// `GetGameRequest` is a structure that represents the form data received when a new game is requested.
// It includes the difficulty level, the color of the player, and the username.
#[derive(FromForm)]
pub struct GetGameRequest{
    // The difficulty level of the game. It is represented as an integer where 1 is easy, 2 is medium, and 3 is hard.
    pub difficulty: i16,
    // The color of the player. It is represented as a character where 'b' is black, 'w' is white, and 'r' is random.
    pub color: char,
    // The username of the player.
    pub username: String
}