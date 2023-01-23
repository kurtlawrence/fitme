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
 c           3.209            0.013     230.3 
──────────────────────────────────────────────
 m           1.770            0.011     149.0 
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
 c           3.209            0.013     230.3 
──────────────────────────────────────────────
 m           1.770            0.011     149.0 
──────────────────────────────────────────────
",
    );
}

#[test]
fn plain() {
    cmd().arg("-o=plain").assert().success().stdout(
        " Parameter  Value  Standard Error  t-value 
 c          3.209           0.013    230.3 
 m          1.770           0.011    149.0 
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
 c          3.209           0.013    230.3 
 m          1.770           0.011    149.0 
",
        );
}

#[test]
fn csv() {
    cmd().arg("-o=csv").assert().success().stdout(
        "\
Parameter,Value,Standard Error,t-value
c,3.2099657167997013,0.013936863525869892,230.32195951702457
m,1.7709542029456211,0.011883297834310212,149.02884936809457
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
c,3.2099657167997013,0.013936863525869892,230.32195951702457
m,1.7709542029456211,0.011883297834310212,149.02884936809457
",
        );
}

#[test]
fn md() {
    cmd().arg("-o=md").assert().success().stdout(
        "\
| Parameter | Value | Standard Error | t-value |
|-----------|-------|----------------|---------|
| c         | 3.209 |          0.013 |   230.3 |
| m         | 1.770 |          0.011 |   149.0 |
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
| c         | 3.209 |          0.013 |   230.3 |
| m         | 1.770 |          0.011 |   149.0 |
",
        );
}

#[test]
fn json() {
    cmd().arg("-o=json").assert().success().stdout(
        "{\"parameter_names\":[\"c\",\"m\"],\"parameter_values\":[3.2099657167997013,1.7709542029456211],\"n\":10,\"xerrs\":[0.013936863525869892,0.011883297834310212],\"rmsr\":0.04392493014188053,\"rsq\":0.9995948974725735,\"tvals\":[230.32195951702457,149.02884936809457]}"
    );

    cmd()
        .arg("-o=json")
        .arg("--no-stats")
        .assert()
        .success()
        .stdout(
        "{\"parameter_names\":[\"c\",\"m\"],\"parameter_values\":[3.2099657167997013,1.7709542029456211],\"n\":10,\"xerrs\":[0.013936863525869892,0.011883297834310212],\"rmsr\":0.04392493014188053,\"rsq\":0.9995948974725735,\"tvals\":[230.32195951702457,149.02884936809457]}"
        );
}
