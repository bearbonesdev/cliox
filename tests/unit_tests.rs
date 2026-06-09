/**
 * these tests test oocli functionality against direct Command initialization
 * rather than command output. The assumption is that if the two commands are
 * "equal" oocli is doing its job. future unit tests may be added to verify this
 * assumption, but uat testing so far confirms.
 */
use oocli::Commandable;

#[derive(Commandable)]
#[oocli(command = "git")]
struct MockCmd {
    #[oocli(pass_through)]
    sub_command: String,
    #[oocli(rename = "m", prefix = "-")]
    message: String,
    #[oocli(skip)]
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
#[oocli(
    command = "git commit",
    env(name = "GIT_COMMITTER_EMAIL", value = "test@example.com")
)]
struct MockEnvCmd {
    #[oocli(rename = "m", prefix = "-")]
    message: String,
    #[oocli(env, rename_all = "SCREAMING_SNAKE_CASE")]
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
#[oocli(command = "git fetch")]
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
#[oocli(command = "git log", delimiter = "=")]
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
#[oocli(command = "git log")]
struct MockLogFieldCmd {
    #[oocli(delimiter = "=")]
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
#[oocli(command = "git fetch", rename_all = "SCREAMING_SNAKE_CASE")]
struct MockRenameAllCmd {
    #[oocli(env)]
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
