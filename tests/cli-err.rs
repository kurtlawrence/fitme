use assert_cmd::Command;

fn cmd() -> Command {
    Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap()
}

#[test]
fn no_args() {
    cmd().assert().failure().stderr(
        "\
error: the following required arguments were not provided:
  <TARGET>
  <EXPR>

Usage: fitme <TARGET> <EXPR> [DATA]

For more information, try \'--help\'.
",
    );
}

#[test]
fn target_not_found() {
    cmd()
        .arg("y_")
        .arg("m * x + c")
        .arg("tests/file1.csv")
        .assert()
        .failure()
        .stderr(
            "\
Error: 
  × in \'tests/file1.csv\'
  ╰─▶ could not find column \'y_\' in headers

",
        );
}

#[test]
fn file_not_found() {
    cmd()
        .arg("y")
        .arg("m * x + c")
        .arg("not-here")
        .assert()
        .failure()
        .stderr(
            "\
Error: 
  × failed to open \'not-here\'
  ╰─▶ No such file or directory (os error 2)

",
        );
}
