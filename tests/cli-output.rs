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
}

#[test]
fn vanilla_no_stats() {
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
}

