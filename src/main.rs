// The `rocket` crate is a web framework for Rust that makes it simple to write fast, secure web applications without sacrificing flexibility, usability, or type safety.
#[macro_use]
extern crate rocket;

// The `utils` module contains all self-written utility function for the game logic
mod utils;


// Importing the public endpoints of our utils
use crate::utils::db::{add_score_entry, DB, get_all, get_top, ScoreEntry, set_score_schema};
use crate::utils::session::{find_session, remove_session, SessionHandler, add_session};
use crate::utils::game::{COLOR, DIFFICULTY, find_with_auto_promotion, Game};
use crate::utils::errors::external::{FenResponse, OkOrResponse, Response};
use crate::utils::requests::GameSettings;

// Importing necessary modules and structures from the `rocket` and `shakmaty` crates.
use rocket_dyn_templates::{context, Template};
use shakmaty::{EnPassantMode, Position};
use rocket::fs::{FileServer, relative};
use rocket::http::{CookieJar, Status};
use rocket::serde::json::{Json};
use rocket::response::Redirect;
use rocket::form::Form;
use shakmaty::fen::Fen;
use shakmaty::uci::Uci;
use rocket::State;

// Route handler for the root URL ("/"). Redirects to "/welcome_page.html"
#[get("/")]
async fn get() -> Redirect {
    Redirect::permanent("/welcome_page.html")
}

// Route handler for "/game". It handles the creation of a new game and its session.
// It takes an optional `new:session` query parameter and `game_settings` form data.
// It uses `CookieJar` to manage session cookies and a `SessionHandler` to manage the game session.
#[post("/game", data="<game_settings>")]
async fn get_game(game_settings: Form<GameSettings>, cookie_jar: &CookieJar<'_>, session_handler: &State<SessionHandler>) -> Response<Template> {
    if let Some(_) = find_session(cookie_jar, session_handler).await {
        // The user already has a session
        if game_settings.new_session.is_none() {
            // The user receives an error, because he was not intentionally requesting a new game
            return Err((Status::BadRequest, "There is already a Session running, please retry!"));
        }
        // The user will receive a new session
        remove_session(cookie_jar, session_handler).await;
    }
    // Creates game instance
    let color = COLOR::new(game_settings.color).ok_or((Status::BadRequest, "Your color submission is invalid"))?;
    let difficulty = DIFFICULTY::new(game_settings.difficulty).ok_or((Status::BadRequest, "Your difficulty submission is invalid"))?;
    let game = Game::new(color.clone(), difficulty.clone(), game_settings.username.clone()).await.ok_or((Status::InternalServerError, "Game could not be created"))?;

    // Add game to the session handler and update cookies
    add_session(game, cookie_jar, session_handler).await;
    // Render game template with provided data
    Ok(Template::render("game", context! {
        username: game_settings.username.clone(),
        difficulty: difficulty.parse_player_name(),
        color: color.parse_code()
    }))
}

// Route handler for "/game_end". It checks if the current game session is over.
// It uses `CookieJar` to manage session cookies and a `SessionHandler` to manage sessions.
#[get("/game_end")]
async fn get_game_end(cookie_jar: &CookieJar<'_>, session_handler: &State<SessionHandler>, db: &State<DB>) -> OkOrResponse<String> {
    // Grabs the users session if it exists
    let session = find_session(cookie_jar, session_handler).await.ok_or((Status::BadRequest, String::from("You are missing a session key")))?;
    let game = session.get().await;

    return if game.board.is_game_over() || game.board.halfmoves() >= 100 {
        let board = &game.board;
        let outcome = board.outcome().ok_or((Status::InternalServerError, String::from("Contradictory game conditions")))?;
        if let Some(winner) = outcome.winner() {
            if game.user_color.to_string().eq(&winner.to_string()[..1]).clone() {
                let score_entry = ScoreEntry::new(&game.username, board.fullmoves().get(), &game.difficulty);
                let conn = &db.get().map_err(|_| (Status::InternalServerError, String::from("Could not add score!")))?;
                add_score_entry(conn, score_entry).map_err(|_| (Status::InternalServerError, String::from("Could not add Score")))?;
            }
        }

        remove_session(cookie_jar, session_handler).await;
        Ok(Status::Ok)
    } else {
        let fen = Fen::from_position(game.board.clone(), EnPassantMode::Legal).to_string();
        Err((Status::NotAcceptable, fen))
    };
}

// Route handler `/move` it handles the players use and the chess engine's response.
// It takes a `mov` alias move as form data representing the players move.
// It uses `CookieJar` to manage session cookies and a `SessionHandler` to manage sessions.
#[post("/move", data = "<mov>")]
async fn post_move(mov: String, cookie_jar: &CookieJar<'_>, session_handler: &State<SessionHandler>) -> FenResponse {
    // Grabs the users session if it exists
    let session = find_session(cookie_jar, session_handler).await.ok_or((Status::BadRequest, String::from("You are missing a session key!")))?;
    let mut game = session.get().await;

    // Makes a temp duplicate of the games fen representation
    let curr_fen = Fen::from_position(game.board.clone(), EnPassantMode::Legal).to_string();

    // Applies user's move, if it is invalid, the current fen will be returned
    let uci: Uci = mov.parse().map_err(move |_| (Status::NotAcceptable, curr_fen))?;
    let mov = find_with_auto_promotion(&uci, &game.board).ok_or((Status::BadRequest, String::from("Your move could not be evaluated!")))?;
    game.board.play_unchecked(&mov);

    // Generates and applies engine's move
    let board_clone = game.board.clone();
    let mov = game.engine.gen_next_move(&board_clone).await.map_err(|_| (Status::InternalServerError, String::from("Could not generate stockfish move")))?;
    game.board.play_unchecked(&mov);

    // Makes duplicate of games fen representation and returns it
    let fen = Fen::from_position(game.board.clone(), EnPassantMode::Legal).to_string();
    Ok(fen)
}

// Route handler `/scoreboard` it returns the top <count> scoreboard entries.
#[get("/scoreboard?<count>")]
async fn get_scoreboard(count: Option<u16>, db: &State<DB>) -> Response<Json<Vec<ScoreEntry>>> {
    let conn = db.get().map_err(|_| (Status::InternalServerError, "Could not access database"))?;
    let scores = match count {
        None => get_all(&conn),
        Some(count) => get_top(&conn, count)
    }.map_err(|_| (Status::InternalServerError, "Could not receive scores!"))?;
    Ok(Json(scores))
}


#[launch]
fn rocket() -> _ {
    // Creates a session handler that stores game states
    let session_handler = SessionHandler::new();
    // Create or open db instance
    let db = DB::new("/db/my.db").expect("Could not open database");
    set_score_schema(&db.get().expect("Could not set schema caused by invalid db")).expect("Error setting schema");
    // Build the rocket application including static file serving, sessions and dynamic html rendering via handlebars
    rocket::build()
        .manage(session_handler)
        .manage(db)
        .mount("/", routes![get_game, post_move, get_game_end, get, get_scoreboard])
        .mount("/", FileServer::from(relative!("/static")))
        .attach(Template::fairing())
}
