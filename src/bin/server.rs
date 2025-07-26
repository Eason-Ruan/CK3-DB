use axum::{
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use serde_json::Value;
use tracing_subscriber;
use clap::Parser;
use derive_more::From;
use human_panic::setup_panic;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde_json;
use std::{
    env, error,
    fmt::{self, Debug, Display, Formatter},
    fs,
    io::{stdin, stdout, IsTerminal},
    ops::Not,
    time::Duration,
};

/// A submodule that provides opaque types commonly used in the project
use ck3_history_extractor::types;

/// A submodule that handles save file parsing
use ck3_history_extractor::parser;
use parser::{process_section, yield_section, GameState, SaveFile, SaveFileError};

/// A submodule that provides objects which are serialized and rendered into HTML.
/// You can think of them like frontend DB view objects into parsed save files.
use ck3_history_extractor::structures;
use structures::{GameObjectDerived, Player};

/// The submodule responsible for creating the [minijinja::Environment] and loading of templates.
use ck3_history_extractor::jinja_env;
use jinja_env::create_env;

/// A module for handling the display of the parsed data.
use ck3_history_extractor::display;
use display::{GetPath, Renderer};

/// A submodule for handling the game data
use ck3_history_extractor::game_data;
use game_data::{GameDataLoader, Localizable};

/// A submodule for handling the arguments passed to the program
use ck3_history_extractor::args;
use args::Args;

/// A submodule for handling Steam integration
use ck3_history_extractor::steam;
use ck3_history_extractor::sql_util;

/// The interval at which the progress bars should update.
const INTERVAL: Duration = Duration::from_secs(1);

/// An error a user has caused. Shame on him.
#[derive(From, Debug)]
enum UserError {
    /// The program is not running in a terminal
    NoTerminal,
    /// The file does not exist
    FileDoesNotExist,
    /// An error occurred during file handling
    FileError(SaveFileError),
}

impl Display for UserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            UserError::NoTerminal => write!(f, "The program is not running in a terminal"),
            UserError::FileDoesNotExist => write!(f, "The file does not exist"),
            UserError::FileError(e) => write!(f, "An error occurred during file handling: {}", e),
        }
    }
}

impl error::Error for UserError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            UserError::FileError(e) => Some(e),
            _ => None,
        }
    }
}

#[tokio::main]
async fn main() {
    // 日志初始化
    tracing_subscriber::fmt::init();

    // 构建路由
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/echo", post(echo_handler))
        .route("/jsonUp", post(game_state_handler))
        .route("/log", get(|| async { "Server is running!" }));


    // 绑定地址
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .await
        .unwrap();
}

async fn root_handler() -> &'static str {
    "Welcome to ck3 rust server!"
}

#[derive(Debug, Deserialize, Serialize)]
struct EchoPayload {
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ApiResponse {
    code: i32,
    message: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ApiResponseWithData<T> {
    code: i32,
    message: String,
    data: T,
}

async fn echo_handler(Json(payload): Json<EchoPayload>) -> Json<EchoPayload> {
    Json(payload)
}

async fn game_state_handler(Json(payload): Json<Value>) -> Json<ApiResponse> {
    // TODO: Implement game state logic here
    let response = ApiResponse {
        code: 200,
        message: "Game state received".to_string(),
    };
    Json(response)
}

async fn log_handler(Json(payload): Json<Value>) {
    // TODO: Implement logging logic here
}
