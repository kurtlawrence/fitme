use clap::{Parser, ValueEnum};
use miette::*;
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};

mod data;
mod expr;
mod solve;

use expr::Equation;

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
}

#[derive(Debug, Copy, Clone, ValueEnum, Default)]
pub enum EquationResolver {
    #[default]
    V1,
}

#[derive(Debug, Copy, Clone, ValueEnum, Default)]
pub enum Output {
    /// Rich table view.
    #[default]
    Table,

    /// Plain, space separated table.
    Plain,
}

impl App {
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
    let data = data::Data::try_from(rdr).wrap_err_with(with_path_ctx)?;
    let fitted = solve::fit(eq, data, &target).wrap_err_with(with_path_ctx)?;

    match out {
        Output::Table => write_table(&fitted, !no_stats).into_diagnostic(),
    }
}

fn write_table(x: &solve::Fit, write_stats: bool) -> io::Result<()> {
    use comfy_table::{Cell, CellAlignment as CA, Row, Table};

    let solve::Fit {
        parameter_names,
        parameter_values,
        n,
        rmsr,
        rsq,
        xerrs,
        tvals,
    } = x;

    let w = &mut io::stdout();

    let mut nfmtr = "[~4]".parse::<numfmt::Formatter>().expect("just fine");

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
            .add_cell(Cell::new(nfmtr.fmt(*v)).set_alignment(CA::Right))
            .add_cell(Cell::new(nfmtr.fmt(*e)).set_alignment(CA::Right))
            .add_cell(Cell::new(nfmtr.fmt(*t)).set_alignment(CA::Right));
        table.add_row(row);
    }

    table.load_preset(comfy_table::presets::UTF8_HORIZONTAL_ONLY);

    writeln!(w, "{table}")?;

    if write_stats {
        writeln!(w, "  Number of observations: {}", nfmtr.fmt(*n as f64))?;
        writeln!(
            w,
            "  Root Mean Squared Residual error: {}",
            nfmtr.fmt(*rmsr)
        )?;
        writeln!(w, "  R-sq Adjusted: {}", nfmtr.fmt(*rsq))?;
    }

    Ok(())
}
