/**
 * these tests test cliox functionality against direct Command initialization
 * rather than command output. The assumption is that if the two commands are
 * "equal" cliox is doing its job. future unit tests may be added to verify this
 * assumption, but uat testing so far confirms.
 */
use cliox::Commandable;

#[derive(Commandable)]
#[cliox(command = "ls")]
struct MockLs {}

/**
 * test basic command implementation
 */
#[test]
fn test_cmd() {
    let expected = std::process::Command::new("ls");

    let actual = std::process::Command::from(MockLs {});

    println!("{:?}", actual);
    assert_eq!(format!("{:?}", expected), format!("{:?}", actual));
}

#[derive(Commandable)]
#[cliox(command = "git")]
struct MockCmd {
    #[cliox(pass_through)]
    sub_command: String,
    #[cliox(rename = "m", prefix = "-")]
    message: String,
    #[cliox(skip)]
    _comment: String,
}

/**
 * test basic command implementation
 */
#[test]
fn test_macro() {
    let msg = "initial commit";

    let mut expected = std::process::Command::new("git");
    expected.arg("commit");
    expected.arg("-m");
    expected.arg(msg);

    let actual = std::process::Command::from(MockCmd {
        sub_command: "commit".to_string(),
        message: msg.to_string(),
        _comment: "this won't end up in the command".to_string(),
    });

    println!("{:?}", actual);
    assert_eq!(format!("{:?}", expected), format!("{:?}", actual));
}

#[derive(Commandable)]
#[cliox(
    command = "git commit",
    env(name = "GIT_COMMITTER_EMAIL", value = "test@example.com")
)]
struct MockEnvCmd {
    #[cliox(rename = "m", prefix = "-")]
    message: String,
    #[cliox(env, rename_all = "SCREAMING_SNAKE_CASE")]
    git_dir: String,
}

#[test]
fn test_env() {
    let msg = "initial commit";

    let mut expected = std::process::Command::new("git");
    expected.arg("commit");
    expected.arg("-m");
    expected.arg(msg);
    expected.env("GIT_COMMITTER_EMAIL", "test@example.com");
    expected.env("GIT_DIR", "~");

    let actual = std::process::Command::from(MockEnvCmd {
        message: msg.to_string(),
        git_dir: "~".to_string(),
    });

    println!("{:?}", actual);
    assert_eq!(format!("{:?}", expected), format!("{:?}", actual));
}

#[derive(Commandable)]
#[cliox(command = "git fetch")]
struct MockFlagCmd {
    prune: Option<bool>,
    all: Option<bool>,
}

#[test]
fn test_flag() {
    let mut expected = std::process::Command::new("git");
    expected.arg("fetch");
    expected.arg("--all");

    let actual = std::process::Command::from(MockFlagCmd {
        prune: None,
        all: Some(true),
    });

    println!("{:?}", actual);
    assert_eq!(format!("{:?}", expected), format!("{:?}", actual));
}

#[derive(Commandable)]
#[cliox(command = "git log", delimiter = "=")]
struct MockLogCmd {
    format: String,
}

#[test]
fn test_log() {
    let mut expected = std::process::Command::new("git");
    expected.arg("log");
    expected.arg("--format=\"%h;%an;%s\"");

    let actual = std::process::Command::from(MockLogCmd {
        format: "\"%h;%an;%s\"".to_string(),
    });

    println!("{:?}", actual);
    assert_eq!(format!("{:?}", expected), format!("{:?}", actual));
}

#[derive(Commandable)]
#[cliox(command = "git log")]
struct MockLogFieldCmd {
    #[cliox(delimiter = "=")]
    format: String,
}

#[test]
fn test_log_field() {
    let mut expected = std::process::Command::new("git");
    expected.arg("log");
    expected.arg("--format=\"%h;%an;%s\"");

    let actual = std::process::Command::from(MockLogFieldCmd {
        format: "\"%h;%an;%s\"".to_string(),
    });

    println!("{:?}", actual);
    assert_eq!(format!("{:?}", expected), format!("{:?}", actual));
}

#[derive(Commandable)]
#[cliox(command = "git fetch", rename_all = "SCREAMING_SNAKE_CASE")]
struct MockRenameAllCmd {
    #[cliox(env)]
    git_dir: String,
}

#[test]
fn test_rename_all() {
    let mut expected = std::process::Command::new("git");
    expected.arg("fetch");
    expected.env("GIT_DIR", "~");

    let actual = std::process::Command::from(MockRenameAllCmd {
        git_dir: "~".to_string(),
    });

    println!("{:?}", actual);
    assert_eq!(format!("{:?}", expected), format!("{:?}", actual));
}
