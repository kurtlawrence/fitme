use assert_cmd::Command;

fn cmd() -> Command {
    let mut c = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    c.args(&["y", "m * x + c", "tests/file1.csv"]);
    c
}

#[test]
fn vanilla() {
    cmd().assert().success().stdout(
        "\
──────────────────────────────────────────────
 Parameter   Value   Standard Error   t-value 
══════════════════════════════════════════════
 m           1.770            0.011     149.0 
──────────────────────────────────────────────
 c           3.209            0.013     230.3 
──────────────────────────────────────────────
  Number of observations: 10.0
  Root Mean Squared Residual error: 0.043
  R-sq Adjusted: 0.999
",
    );

    cmd().arg("--no-stats").assert().success().stdout(
        "\
──────────────────────────────────────────────
 Parameter   Value   Standard Error   t-value 
══════════════════════════════════════════════
 m           1.770            0.011     149.0 
──────────────────────────────────────────────
 c           3.209            0.013     230.3 
──────────────────────────────────────────────
",
    );
}

#[test]
fn plain() {
    cmd().arg("-o=plain").assert().success().stdout(
        " Parameter  Value  Standard Error  t-value 
 m          1.770           0.011    149.0 
 c          3.209           0.013    230.3 
  Number of observations: 10.0
  Root Mean Squared Residual error: 0.043
  R-sq Adjusted: 0.999
",
    );

    cmd()
        .arg("-o=plain")
        .arg("--no-stats")
        .assert()
        .success()
        .stdout(
            " Parameter  Value  Standard Error  t-value 
 m          1.770           0.011    149.0 
 c          3.209           0.013    230.3 
",
        );
}

#[test]
fn csv() {
    cmd().arg("-o=csv").assert().success().stdout(
        "\
Parameter,Value,Standard Error,t-value
m,1.7709542026136489,0.011883297869391124,149.0288489002076
c,3.209965716831507,0.013936863563326086,230.32195890030198
  Number of observations: 10.0
  Root Mean Squared Residual error: 0.043
  R-sq Adjusted: 0.999
",
    );

    cmd()
        .arg("-o=csv")
        .arg("--no-stats")
        .assert()
        .success()
        .stdout(
            "\
Parameter,Value,Standard Error,t-value
m,1.7709542026136489,0.011883297869391124,149.0288489002076
c,3.209965716831507,0.013936863563326086,230.32195890030198
",
        );
}

#[test]
fn md() {
    cmd().arg("-o=md").assert().success().stdout(
        "\
| Parameter | Value | Standard Error | t-value |
|-----------|-------|----------------|---------|
| m         | 1.770 |          0.011 |   149.0 |
| c         | 3.209 |          0.013 |   230.3 |
  Number of observations: 10.0
  Root Mean Squared Residual error: 0.043
  R-sq Adjusted: 0.999
",
    );

    cmd()
        .arg("-o=md")
        .arg("--no-stats")
        .assert()
        .success()
        .stdout(
            "\
| Parameter | Value | Standard Error | t-value |
|-----------|-------|----------------|---------|
| m         | 1.770 |          0.011 |   149.0 |
| c         | 3.209 |          0.013 |   230.3 |
",
        );
}

#[test]
fn json() {
    cmd().arg("-o=json").assert().success().stdout(
        "{\"parameter_names\":[\"m\",\"c\"],\"parameter_values\":[1.7709542026136489,3.209965716831507],\"n\":10,\"xerrs\":[0.011883297869391124,0.013936863563326086],\"rmsr\":0.04392493014188046,\"rsq\":0.9995948974724216,\"tvals\":[149.0288489002076,230.32195890030198]}",
    );

    cmd()
        .arg("-o=json")
        .arg("--no-stats")
        .assert()
        .success()
        .stdout(
        "{\"parameter_names\":[\"m\",\"c\"],\"parameter_values\":[1.7709542026136489,3.209965716831507],\"n\":10,\"xerrs\":[0.011883297869391124,0.013936863563326086],\"rmsr\":0.04392493014188046,\"rsq\":0.9995948974724216,\"tvals\":[149.0288489002076,230.32195890030198]}",
        );
}
