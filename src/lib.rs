/// A submodule that handles save file parsing
pub mod parser;

/// A submodule that handles save file parsing
pub mod types;

/// A submodule that provides objects which are serialized and rendered into HTML.
/// You can think of them like frontend DB view objects into parsed save files.
pub mod structures;

/// The submodule responsible for creating the [minijinja::Environment] and loading of templates.
pub mod jinja_env;

/// A module for handling the display of the parsed data.
pub mod display;

/// A submodule for handling the game data
pub mod game_data;

/// A submodule for handling the arguments passed to the program
pub mod args;

/// A submodule for handling Steam integration
pub mod steam;
pub mod sql_util;