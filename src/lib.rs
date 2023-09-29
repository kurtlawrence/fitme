//! CLI curve fitting tool. Parameterise an equation from a CSV dataset.
//!
//! `fitme` is primarily a CLI tool. For usage and examples, see the [repository
//! README](https://github.com/kurtlawrence/fitme).
//!
//! `fitme` _can_ be used as a library, the exposed API is a minimal set required for use of
//! the [`fit`] function.
//! _If using as a library, see the [`fit`] function's documentation for an example._
#![warn(missing_docs)]

use clap::{Parser, ValueEnum};
use miette::*;
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};

mod data;
pub mod expr;
mod solve;

pub use data::{Data, DataRow, Headers};
pub use expr::Equation;
pub use solve::{fit, Fit};

/// CLI curve fitting tool.
/// Parameterise an equation from a CSV dataset.
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct App {
    /// The target column (the Y value).
    pub target: String,

    /// The parameterised equation.
    pub expr: String,

    /// Path to input CSV file.
    /// If left blank, stdin is read.
    pub data: Option<PathBuf>,

    /// The version of equation resolver to use.
    #[arg(long, default_value_t, value_enum)]
    pub eq_resolver: EquationResolver,

    /// The output format to write to stdout.
    #[arg(short, long, default_value_t, value_enum)]
    pub out: Output,

    /// Do not output the fitting statistics along with parameters.
    #[arg(short, long)]
    pub no_stats: bool,

    /// Output debug information about the expression and input data.
    /// Does not attempt a fit.
    #[arg(long)]
    pub debug: bool,
}

/// Versions of the equation resolver.
#[derive(Debug, Copy, Clone, ValueEnum, Default)]
pub enum EquationResolver {
    /// Version #1.
    #[default]
    V1,
}

/// How do you want the output formatted?
#[derive(Debug, Copy, Clone, ValueEnum, Default)]
pub enum Output {
    /// Rich table view.
    #[default]
    Table,

    /// Plain, space separated table.
    Plain,

    /// Comma separated value output.
    Csv,

    /// Markdown formatted table.
    Md,

    /// Serialised structure of fitted parameters.
    Json,
}

impl App {
    /// Fit data and output results.
    pub fn run(self) -> Result<()> {
        match self.eq_resolver {
            EquationResolver::V1 => run::<expr::v1::Eq>(self),
        }
    }
}

fn run<E>(app: App) -> Result<()>
where
    E: Equation,
{
    let App {
        target,
        expr,
        data,
        eq_resolver: _,
        out,
        no_stats,
        debug,
    } = app;

    let mut rdr = match &data {
        Some(path) => data::CsvReader::new(io::BufReader::new(
            fs::File::open(path)
                .into_diagnostic()
                .wrap_err_with(|| format!("failed to open '{}'", path.display()))?,
        )),
        None => {
            eprintln!("Reading CSV from stdin");
            data::CsvReader::new(io::stdin())
        }
    };

    let with_path_ctx = || {
        data.as_ref()
            .map(|p| format!("in '{}'", p.display()))
            .unwrap_or_else(|| "from stdin".into())
    };

    let hdrs = rdr.headers().wrap_err_with(with_path_ctx)?;
    let eq = E::parse(&expr, hdrs).wrap_err_with(with_path_ctx)?;

    if debug {
        return output_debug(&eq, hdrs, &target);
    }

    let data = data::Data::try_from(rdr).wrap_err_with(with_path_ctx)?;
    let fitted = fit(eq, data, &target).wrap_err_with(with_path_ctx)?;

    match out {
        Output::Table => write_rich_table(&fitted, !no_stats),
        Output::Plain => write_plain_table(&fitted, !no_stats),
        Output::Csv => write_csv_table(&fitted, !no_stats).into_diagnostic(),
        Output::Md => write_md_table(&fitted, !no_stats),
        Output::Json => write_json_table(&fitted),
    }
}

fn nfmtr() -> numfmt::Formatter {
    "[~4]".parse::<numfmt::Formatter>().expect("just fine")
}

fn write_rich_table(x: &Fit, write_stats: bool) -> Result<()> {
    write_table(x, write_stats, comfy_table::presets::UTF8_HORIZONTAL_ONLY).into_diagnostic()
}

fn write_plain_table(x: &Fit, write_stats: bool) -> Result<()> {
    write_table(x, write_stats, comfy_table::presets::NOTHING).into_diagnostic()
}

fn write_csv_table(x: &Fit, write_stats: bool) -> io::Result<()> {
    let Fit {
        parameter_names,
        parameter_values,
        n,
        rmsr,
        rsq,
        xerrs,
        tvals,
    } = x;

    let mut nfmtr = nfmtr();

    let mut stdout = io::stdout();

    let mut w = csv::Writer::from_writer(&mut stdout);

    w.write_record(["Parameter", "Value", "Standard Error", "t-value"])?;

    for (((p, v), e), t) in parameter_names
        .iter()
        .zip(parameter_values)
        .zip(xerrs)
        .zip(tvals)
    {
        w.write_field(p)?;
        w.write_field(v.to_string())?;
        w.write_field(e.to_string())?;
        w.write_field(t.to_string())?;
        w.write_record(None::<&[u8]>)?;
    }

    drop(w);

    if write_stats {
        writeln!(&mut stdout, "  Number of observations: {}", nfmtr.fmt2(*n))?;
        writeln!(
            &mut stdout,
            "  Root Mean Squared Residual error: {}",
            nfmtr.fmt2(*rmsr)
        )?;
        writeln!(&mut stdout, "  R-sq Adjusted: {}", nfmtr.fmt2(*rsq))?;
    }

    Ok(())
}

fn write_md_table(x: &Fit, write_stats: bool) -> Result<()> {
    write_table(x, write_stats, comfy_table::presets::ASCII_MARKDOWN).into_diagnostic()
}

fn write_table(x: &Fit, write_stats: bool, table_fmt: &str) -> io::Result<()> {
    use comfy_table::{Cell, CellAlignment as CA, Row, Table};

    let Fit {
        parameter_names,
        parameter_values,
        n,
        rmsr,
        rsq,
        xerrs,
        tvals,
    } = x;

    let w = &mut io::stdout();

    let mut nfmtr = nfmtr();

    let mut table = Table::new();

    table.set_header(["Parameter", "Value", "Standard Error", "t-value"]);

    for (((p, v), e), t) in parameter_names
        .iter()
        .zip(parameter_values)
        .zip(xerrs)
        .zip(tvals)
    {
        let mut row = Row::new();
        row.add_cell(Cell::new(p))
            .add_cell(Cell::new(nfmtr.fmt2(*v)).set_alignment(CA::Right))
            .add_cell(Cell::new(nfmtr.fmt2(*e)).set_alignment(CA::Right))
            .add_cell(Cell::new(nfmtr.fmt2(*t)).set_alignment(CA::Right));
        table.add_row(row);
    }

    table.load_preset(table_fmt);

    writeln!(w, "{table}")?;

    if write_stats {
        writeln!(w, "  Number of observations: {}", nfmtr.fmt2(*n))?;
        writeln!(
            w,
            "  Root Mean Squared Residual error: {}",
            nfmtr.fmt2(*rmsr)
        )?;
        writeln!(w, "  R-sq Adjusted: {}", nfmtr.fmt2(*rsq))?;
    }

    Ok(())
}

fn write_json_table(x: &Fit) -> Result<()> {
    serde_json::to_writer(io::stdout(), x).into_diagnostic()
}

fn output_debug<E: Equation>(eq: &E, hdrs: &Headers, target: &str) -> Result<()> {
    if let Some(expr) = eq.expr() {
        println!("✖️ Expression:");
        println!("  {expr}");
    }

    let params = eq.params();
    println!("📊 Parameters:");
    if params.is_empty() {
        println!("  <none>");
    } else {
        for p in params {
            print!("  {p}");
            let h = data::match_hdr_help(hdrs, &p);
            if !h.starts_with("help - no columns match") {
                println!(" :: {h}");
            } else {
                println!();
            }
        }
    }

    let vars = eq.vars();
    println!("🧮 Variables:");
    if vars.is_empty() {
        println!("  <none>");
    } else {
        for x in vars {
            println!("  {x}");
        }
    }

    println!("🔎 Target:");
    println!("  {target}");

    hdrs.find_ignore_case_and_ws(target)
        .ok_or_else(|| miette!("target column '{}' not found in headers", target))
        .wrap_err_with(|| data::match_hdr_help(hdrs, target))?;

    Ok(())
}
