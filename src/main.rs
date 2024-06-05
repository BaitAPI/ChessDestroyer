#[macro_use]
extern crate rocket;

mod utils;

use rocket::form::Form;
use rocket::fs::{FileServer, relative};
use rocket::http::{CookieJar, Status};
use rocket::response::Redirect;
use rocket::State;
use rocket_dyn_templates::{context, Template};
use shakmaty::{EnPassantMode, Position};
use crate::utils::game::{COLOR, DIFFICULTY, Game};
use crate::utils::requests::{GetGameRequest};
use crate::utils::session::{find_session, remove_session, SessionHandler, add_session};
use shakmaty::fen::Fen;
use shakmaty::uci::Uci;

#[get("/")]
async fn get() -> Redirect {
    Redirect::to("/welcome_page.html")
}

#[get("/game?<new_session>", data = "<game_settings>")]
async fn get_game(new_session: Option<bool>, game_settings: Form<GetGameRequest>, cookie_jar: &CookieJar<'_>, session_handler: &State<SessionHandler>) -> Result<Template, (Status, &'static str)> {
    if let Some(_) = find_session(cookie_jar, session_handler).await {
        if new_session.is_none() {
            return Err((Status::BadRequest, "There is already a Session running, please retry!"));
        }
        remove_session(cookie_jar, session_handler).await;
    }
    let color = COLOR::new(game_settings.color).ok_or((Status::BadRequest, "Your color submission is invalid"))?;
    let difficulty = DIFFICULTY::new(game_settings.difficulty).ok_or((Status::BadRequest, "Your difficulty submission is invalid"))?;
    let game = Game::new(color.clone(), difficulty.clone(), game_settings.username.clone()).await.ok_or((Status::InternalServerError, "Game could not be created"))?;

    add_session(game, cookie_jar, session_handler).await;
    Ok(Template::render("game", context! {
        username: game_settings.username.clone(),
        difficulty: difficulty.parse_player_name(),
        color: color.parse_code()
    }))
}

#[get("/game_end")]
async fn get_game_end(cookie_jar: &CookieJar<'_>, session_handler: &State<SessionHandler>) -> Result<Status, (Status, String)> {
    let session = find_session(cookie_jar, session_handler).await.ok_or((Status::BadRequest, String::from("You are missing a session key")))?;
    let game = session.get().await;

    return if game.board.is_game_over() {
        Ok(Status::Ok)
    } else {
        let fen = Fen::from_position(game.board.clone(), EnPassantMode::Legal).to_string();
        Err((Status::NotAcceptable, fen))
    };
}

#[post("/move", data = "<mov>")]
async fn post_move(mov: String, cookie_jar: &CookieJar<'_>, session_handler: &State<SessionHandler>) -> Result<String, (Status, String)> {
    let session = find_session(cookie_jar, session_handler).await.ok_or((Status::BadRequest, String::from("You are missing a session key!")))?;
    let mut game = session.get().await;

    let curr_fen = Fen::from_position(game.board.clone(), EnPassantMode::Legal).to_string();

    let mov: Uci = mov.parse().map_err(|_| (Status::NotAcceptable, curr_fen.clone()))?;
    let mov = mov.to_move(&game.board).map_err(|_| (Status::NotAcceptable, curr_fen.clone()))?;
    game.board.play_unchecked(&mov);

    let board_clone = game.board.clone();
    let mov = game.engine.gen_next_move(&board_clone).await.map_err(|_| (Status::InternalServerError, String::from("Could not generate stockfish move")))?;
    game.board.play_unchecked(&mov);

    let fen = Fen::from_position(game.board.clone(), EnPassantMode::Legal).to_string();
    Ok(fen)
}


#[launch]
fn rocket() -> _ {
    let session_handler = SessionHandler::new();
    rocket::build()
        .manage(session_handler)
        .mount("/", routes![get_game, post_move, get_game_end, get])
        .mount("/", FileServer::from(relative!("/static")))
        .attach(Template::fairing())
}