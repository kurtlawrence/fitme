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
  ├─▶ help - these headers are similar: y
  ╰─▶ could not find column \'y_\' in headers

",
        );

    cmd()
        .arg("a space ")
        .arg("m * x + c")
        .arg("tests/file1.csv")
        .assert()
        .failure()
        .stderr(
            "\
Error: 
  × in \'tests/file1.csv\'
  ├─▶ help - these headers are similar: aSpacecol
  ╰─▶ could not find column \'a space \' in headers

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

#[test]
fn matching_column_name() {
    cmd()
        .arg("aSpaceCol")
        .arg("10 - x - y + FOO")
        .arg("tests/file1.csv")
        .assert()
        .success()
        .stdout(
            "\
───────────────────────────────────────────────
 Parameter   Value    Standard Error   t-value 
═══════════════════════════════════════════════
 FOO         -1.024            1.641    -0.624 
───────────────────────────────────────────────
  Number of observations: 10.0
  Root Mean Squared Residual error: 5.191
  R-sq Adjusted: 0.243
",
        );
}

#[test]
fn zero_params() {
    cmd()
        .arg("y")
        .arg("2 * x")
        .arg("tests/file1.csv")
        .assert()
        .failure()
        .stderr(
            "\
Error: 
  × in \'tests/file1.csv\'
  ├─▶ supplied expr: 2 * x
  ├─▶ equation must have a least one variable which does not match a column
  │   header
  ╰─▶ equation has 0 parameters to fit

",
        );
}

#[test]
fn invalid_expr() {
    cmd()
        .arg("y")
        .arg("3 * 2x +")
        .arg("tests/file1.csv")
        .assert()
        .failure()
        .stderr(
            "\
Error: 
  × in \'tests/file1.csv\'
  ├─▶ parsing \'3 * 2x +\' failed
  ╰─▶ Parse error: Unexpected token at byte 5.

",
        );
}

#[test]
fn invalid_csv() {
    cmd()
        .arg("y")
        .arg("3 * 2 * x + b")
        .arg("tests/file2.csv")
        .assert()
        .failure()
        .stderr(
            "\
Error: 
  × in \'tests/file2.csv\'
  ├─▶ in row index 2
  ├─▶ in column index 1
  ╰─▶ failed to parse \'bar\' as number

",
        );
}

#[test]
fn supported_math() {
    cmd()
        .arg("y")
        .arg("sin(x) + ln(x) + cos(x) + tan(x) + log(x) + sqrt(x) + exp(x) + abs(x)")
        .arg("tests/file1.csv")
        .assert()
        .failure()
        .stderr(
            "\
Error: 
  × in \'tests/file1.csv\'
  ├─▶ supplied expr: sin(x) + ln(x) + cos(x) + tan(x) + log(x) + sqrt(x) +
  │   exp(x) + abs(x)
  ├─▶ equation must have a least one variable which does not match a column
  │   header
  ╰─▶ equation has 0 parameters to fit

",
        );
}
