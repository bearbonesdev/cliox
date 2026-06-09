# cliox

`cliox` is a lightweight procedural macro designed to wrap existing command line
interfaces with Rust: `cli oxidation` => `cliox`.

in an effort to remain familiar, much of the syntax is ~~stolen~~ borrowed from
[serde](https://github.com/serde-rs/serde) -- as well as much of their implementation of `rename_all`.

## use
using `cliox` is very simple, all you have to do is derive `Commandable` and supply the base command

```rust
	#[derive(Commandable)]
	#[cliox(command="ls")]
	struct Ls{}

	// usage
	let cmd = std::process::Command::from(Ls{});
```

in the `command` attribute, you can denote subcommands/static arguments with simple space delimiting:

```rust
	#[derive(Commandable)]
	#[cliox(command="git fetch")]
	struct GitFetch{}

	// usage
	let cmd = std::process::Command::from(GitFetch{});
```

struct fields easily map to flags, subcommands, or env with simple attributes

```rust
	#[derive(Commandable)]
	#[cliox(command="git")]
	struct Git{
		#[cliox(pass_through)]
		sub_command: String,
		#[cliox(env, rename_all="SCREAMING_SNAKE_CASE")]
		git_dir: Option<String>,
		#[cliox(rename="m",prefix="-")]
		message: String,
		#[cliox(skip)]
		meta: String,
	}

	// usage
	let cmd = std::process::Command::from(Git{
			sub_command:"commit".to_string(),
			git_dir: None,
			message: "initial commit",
			meta: "not included in the command",
	});
```
