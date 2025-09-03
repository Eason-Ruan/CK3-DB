use derive_more::From;
use std::{
    error,
    fmt::{self, Debug, Display, Formatter},
    time::Duration,
};
use std::path::PathBuf;
use jomini::common::Date;
use sqlx::Row;
use crate::game_data::{GameDataLoader, Localizable};
use crate::parser::{process_section, yield_section, GameState, SaveFile, SaveFileError};
use crate::structures::{date_to_native_date, GameObjectDerived, Player};
use crate::types::Wrapper;
// A submodule that provides opaque types commonly used in the project


/// An error a user has caused. Shame on him.
#[derive(From, Debug)]
pub enum UserError {
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

/// Main function. This is the entry point of the program.
///
/// # Process
///
/// 1. Reads user input through the command line arguments or prompts the user for input.
/// 2. Parses the save file.
///     1. Initializes a [SaveFile] object using the provided file name
///     2. Iterates over the Section objects in the save file
///         If the section is of interest to us (e.g. `living`, `dead_unprunable`, etc.):
///         1. We parse the section into [SaveFileObject](crate::parser::SaveFileObject) objects
///         2. We parse the objects into [Derived](structures::GameObjectDerived) objects
///         3. We store the objects in the [GameState] object
/// 3. Initializes a [minijinja::Environment] and loads the templates from the `templates` folder
/// 4. Foreach encountered [structures::Player] in game:
///     1. Creates a folder with the player's name
///     2. Renders the objects into HTML using the templates and writes them to the folder
/// 5. Prints the time taken to parse the save file
///
pub async fn data_proc(filename: &PathBuf, include_paths: &Vec<PathBuf>, language: &'static str, pool: &sqlx::SqlitePool) -> Result<(), UserError> {
    let save_filename = filename.file_name()
        .and_then(|name| name.to_str())
        .unwrap()
        .to_string();
    if file_is_inserted(pool, &save_filename).await.unwrap() {
        return Ok(());
    }
    let mut loader = GameDataLoader::new(false, language);
    for path in include_paths.iter().rev() {
        loader.process_path(path).unwrap();
    }
    let mut data = loader.finalize();
    //initialize the save file
    let save = SaveFile::open(filename)?;
    // this is sort of like the first round of filtering where we store the objects we care about
    let mut game_state: GameState = GameState::default();
    let mut players: Vec<Player> = Vec::new();
    let mut tape = save.tape();
    while let Some(res) = yield_section(&mut tape) {
        let mut section = res.unwrap();
        // if an error occured somewhere here, there's nothing we can do
        process_section(&mut section, &mut game_state, &mut players).unwrap();
    }
    //prepare things for rendering
    game_state.localize(&mut data).unwrap();
    let id = meta_data_insert(pool, &save_filename, language,game_state.get_current_date().unwrap(), game_state.get_offset_date().unwrap()).await.unwrap();
    game_state.sql_export_game_state(pool, id).await.unwrap();
    players_insert(pool, id, &players).await.unwrap();
    Ok(())
}

async fn meta_data_insert(pool: &sqlx::SqlitePool, filename: &str, language: &str, current_date: Date, offset_date: Date) -> Result<i64, sqlx::Error> {
    let res = sqlx::query(r#"
            INSERT OR IGNORE INTO game_metadata(save_file_name, language, current_date, offset_date)
            VALUES (?, ?, ?, ?)
            RETURNING id
        "#)
        .bind(filename)
        .bind(language)
        .bind(date_to_native_date(&current_date))
        .bind(date_to_native_date(&offset_date))
        .fetch_optional(pool)
        .await?;
    let id: i64 = res.unwrap().get("id");
    Ok(id)
}

async fn file_is_inserted(pool: &sqlx::SqlitePool, filename: &str) -> Result<bool, sqlx::Error> {
    let res = sqlx::query(r#"
        SELECT 1 FROM game_metadata WHERE save_file_name = ?
        "#)
            .bind(filename)
            .fetch_optional(pool)
            .await?;
    Ok(res.is_some())
}

async fn players_insert(pool: &sqlx::SqlitePool, meta_id: i64, players: &Vec<Player>) -> Result<(), sqlx::Error> {
    for player in players {
        let res = sqlx::query(r#"
            INSERT OR IGNORE INTO players(player_name, curr_character_id, meta_id)
            VALUES(?, ?, ?)
            RETURNING id
        "#)
            .bind(player.get_name().to_string())
            .bind(player.get_character().as_ref().map(|c| c.get_internal().get_id() as i64))
            .bind(meta_id)
            .fetch_optional(pool)
            .await?;
        let id: i64 = res.unwrap().get("id");
        for node in player.get_lineage().iter() {
            sqlx::query(r#"
                INSERT OR IGNORE INTO lineages(character_id, player_id, date, score, prestige, piety, dread, lifestyle, meta_id)
                VALUES(?,?, ?, ?, ?, ?, ?, ?, ?)
                RETURNING id
            "#)
                .bind(node.get_character().get_internal().get_id() as i64)
                .bind(id)
                .bind(date_to_native_date(&node.get_date()))
                .bind(node.get_score())
                .bind(node.get_prestige())
                .bind(node.get_piety())
                .bind(node.get_dread() as f64)
                .bind(node.get_lifestyle().map(|f| f.to_string()))
                .bind(meta_id)
                .fetch_optional(pool)
                .await?;
        }
    }
    Ok(())
}