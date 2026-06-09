//! # cliox
//!
//! `cliox` is a lightweight framework designed to wrap existing command line
//! interfaces with Rust. It provides macros for annotating field structs to
//! enable easy declaration of a command, subcommands, flags, arguments, and
//! environmental variables.
//!
//! ## Examples
//!
//! Basic example:
//! ```rust
//! #[derive(Commandable)]
//! #[cliox(command="git fetch")]
//! struct GitFetch{}
//!
//! // usage
//! let cmd = std::process::Command::from(GitFetch{});
//! ```
//!
//! Example with multiple fields:
//! ```rust
//! #[derive(Commandable)]
//!	#[cliox(command="git")]
//!	struct Git{
//!	    #[cliox(pass_through)]
//!     sub_command: String,
//!     #[cliox(env, rename_all="SCREAMING_SNAKE_CASE")]
//!     git_dir: Option<String>,
//!	    #[cliox(rename="m",prefix="-")]
//!	    message: String,
//!	    #[cliox(skip)]
//!	    meta: String,
//!	}
//!
//! // usage
//!	let cmd = std::process::Command::from(Git{
//!	    sub_command:"commit".to_string(),
//!	    git_dir: None,
//!	    message: "initial commit",
//!	    meta: "not included in the command",
//!	});
//! ```
//!
//!
////////////////////////////////////////////////////////////////////////////////
pub use cliox_macros::Commandable;
