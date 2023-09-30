use super::*;
use data::Data;
use rmpfit::{MPError, MPFitter, MPResult};
use serde::*;

/// The result of [`fit`].
#[derive(Serialize, Deserialize)]
pub struct Fit {
    /// The names of the parameters.
    pub parameter_names: Vec<String>,
    /// The fitted values of the parameters.
    pub parameter_values: Vec<f64>,

    /// Number of observations.
    pub n: u64,

    /// The Standard Error of each parameter.
    pub xerrs: Vec<f64>,

    /// Root Mean Squared Residual error.
    pub rmsr: f64,

    /// Adjusted R squared value.
    pub rsq: f64,

    /// Each parameters t-value.
    pub tvals: Vec<f64>,
}

impl Fit {
    /// Write the results of the fit in a particular output format to a writer.
    pub fn write_results<W: std::io::Write>(
        &self,
        output: Output,
        write_statistics: bool,
        wtr: W,
    ) -> Result<()> {
        match output {
            Output::Table => write_rich_table(self, write_statistics, wtr),
            Output::Plain => write_plain_table(self, write_statistics, wtr),
            Output::Csv => write_csv_table(self, write_statistics, wtr).into_diagnostic(),
            Output::Md => write_md_table(self, write_statistics, wtr),
            Output::Json => write_json_table(self),
        }
    }
}

struct Fitter<E> {
    data: Data,
    eq: E,
    tgt: usize,
}

/// Fit an equation using the input data.
///
/// If you are using `fitme` as a library, this is the function to use!
///
/// ## Equation
/// The equation is anything which implements [`Equation`].
/// See [`crate::expr::v1::Eq`] for an implementation.
///
/// ## Target
/// Target is the observed resulting column. For example, in the equation `y = mx + c`, `y` would
/// be the target column.
///
/// # Example
/// Let's fit a linear regression to the following data:
///
/// |  y  |  x  |
/// | --- | --- |
/// | 1.9000429E-01 | -1.7237128E+00 |
/// | 6.5807428E+00 | 1.8712276E+00 |
/// | 1.4582725E+00 | -9.6608055E-01 |
/// | 2.7270851E+00 | -2.8394297E-01 |
/// | 5.5969253E+00 | 1.3416969E+00 |
/// | 5.6249280E+00 | 1.3757038E+00 |
/// | 0.787615 | -1.3703436E+00 |
/// | 3.2599759E+00 | 4.2581975E-02 |
/// | 2.9771762E+00 | -1.4970151E-01 |
/// | 4.5936475E+00 | 8.2065094E-01 |
///
/// Equation: `y = m * x + c`
///
/// Here the:
/// - _target_: `y`
/// - _variables_: `x`
/// - _parameters_: `m`, `c`
///
/// ```rust
/// use fitme::*;
///
/// let data = Data::new(
///     Headers::from_iter(["y", "x"]),
///     vec![
///         vec![1.9000429E-01,-1.7237128E+00],
///         vec![6.5807428E+00,1.8712276E+00],
///         vec![1.4582725E+00,-9.6608055E-01],
///         vec![2.7270851E+00,-2.8394297E-01],
///         vec![5.5969253E+00,1.3416969E+00],
///         vec![5.6249280E+00,1.3757038E+00],
///         vec![0.787615,-1.3703436E+00],
///         vec![3.2599759E+00,4.2581975E-02],
///         vec![2.9771762E+00,-1.4970151E-01],
///         vec![4.5936475E+00,8.2065094E-01],
///     ]
/// ).unwrap();
///
/// let eq = fitme::expr::v1::Eq::parse("m * x + c", data.headers()).unwrap();
///
/// let fit = fitme::fit(eq, data, "y").unwrap();
///
/// assert_eq!(fit.n, 10);
/// assert_eq!(&fit.parameter_names, &["c".to_string(), "m".to_string()]);
/// assert_eq!(&fit.parameter_values, &[3.2099657167997013, 1.7709542029456211]);
/// ```
pub fn fit<E: Equation>(eq: E, data: Data, target: &str) -> Result<Fit> {
    let tgt = data
        .headers()
        .find_ignore_case_and_ws(target)
        .ok_or_else(|| miette!("could not find column '{}' in headers", target))
        .wrap_err_with(|| data::match_hdr_help(data.headers(), target))?;

    ensure_float_values_in_data(&eq, &data, tgt)?;

    let fitter = Fitter { data, eq, tgt };

    // we try to guess a set of params that can work
    let mut params =
        guess_params(&fitter.data, &fitter.eq).unwrap_or_else(|| vec![0.1; fitter.eq.params_len()]);

    if params.is_empty() {
        let mut x = Err(miette!("equation has 0 parameters to fit")).wrap_err(
            "equation must have a least one variable which does not match a column header",
        );
        if let Some(e) = fitter.eq.expr() {
            x = x.wrap_err_with(|| format!("supplied expr: {e}"));
        }

        return x;
    }

    let config = rmpfit::MPConfig {
        max_iter: 3000,
        ..Default::default()
    };

    let status = fitter
        .mpfit(&mut params, None, &config)
        .map_err(|e| miette!("{}", e))
        .wrap_err("failed to fit the equation to the input data")?;

    let Fitter { data, eq, tgt } = fitter;

    let n = data.len() as f64;
    let k = params.len() as f64;

    let mean_y = data
        .rows()
        .map(|row| row.get_num(tgt).expect("inside data").expect("is number"))
        .sum::<f64>()
        / n;

    // Y predicition from regression.
    let y_pred: Vec<f64> = data
        .rows()
        .map(|row| eq.solve(&params, row))
        .try_fold(Vec::new(), |mut x, y| {
            y.map(|y| {
                x.push(y);
                x
            })
        })
        .ok_or_else(|| miette!("failed to solve equation when summarising"))?;

    // Degrees of Freedom Residual
    let dfr = n - k - 1.;

    // Sum of Square Residuals
    let ssr = data
        .rows()
        .zip(&y_pred)
        .map(|(row, y_)| row.get_num(tgt).expect("inside data").expect("is number") - y_)
        .map(|x| x.powi(2))
        .sum::<f64>();

    // Sum of Squares Explained
    let sse = y_pred
        .into_iter()
        .map(|y| y - mean_y)
        .map(|x| x.powi(2))
        .sum::<f64>();

    // Root Mean Squared Residual
    let rmsr = (ssr / dfr).sqrt();

    // Sum of Squares Total
    let sst = sse + ssr;

    let rsq = 1. - ssr / sst;

    // Adjusted R squared.
    let rsq = 1. - (1. - rsq) * (n - 1.) / dfr;

    // rmpfit seems to give the sqrt of the Cjj number.
    // multiplying this by the rmsr gives a std error which matches R lm function
    let xerrs = status
        .xerror
        .into_iter()
        .map(|x| x * rmsr)
        .collect::<Vec<_>>();

    let tvals = params
        .iter()
        .zip(&xerrs)
        .map(|(co, er)| co / er)
        .collect::<Vec<_>>();

    Ok(Fit {
        parameter_names: eq.params(),
        parameter_values: params,
        n: data.len() as u64,
        xerrs,
        rmsr,
        rsq,
        tvals,
    })
}

impl<E: Equation> MPFitter for Fitter<E> {
    fn number_of_points(&self) -> usize {
        self.data.len()
    }

    fn eval(&self, params: &[f64], deviates: &mut [f64]) -> MPResult<()> {
        for (d, row) in deviates.iter_mut().zip(self.data.rows()) {
            let f = self.eq.solve(params, row).ok_or(MPError::Eval)?;

            if f.is_finite() {
                let y = row
                    .get_num(self.tgt)
                    .expect("inside data")
                    .expect("is number");
                *d = y - f;
            } else {
                *d = 1e13; // very large deviation
            }
        }

        Ok(())
    }
}

fn ensure_float_values_in_data<E: Equation>(eq: &E, data: &Data, tgt: usize) -> Result<()> {
    fn chk_col(d: &Data, c: usize) -> Result<()> {
        for r in d.rows() {
            r.get_num(c)
                .ok_or_else(|| miette!("column index {} not in table", c))??;
        }
        Ok(())
    }

    chk_col(data, tgt)?;

    for p in eq.vars() {
        let c = data
            .headers()
            .find_ignore_case_and_ws(&p)
            .ok_or_else(|| miette!("could not find column '{}' in headers", p))
            .wrap_err_with(|| data::match_hdr_help(data.headers(), &p))?;
        chk_col(data, c)?;
    }

    Ok(())
}

fn guess_params<E: Equation>(data: &Data, eq: &E) -> Option<Vec<f64>> {
    let r = data.rows().next()?;
    let mut ps = vec![0.0; eq.params_len()];

    if eq.solve(&ps, r).map(|x| x.is_finite()).unwrap_or_default() {
        return Some(ps);
    }

    ps.fill(1.0);
    if eq.solve(&ps, r).map(|x| x.is_finite()).unwrap_or_default() {
        return Some(ps);
    }

    ps.fill(0.5);
    if eq.solve(&ps, r).map(|x| x.is_finite()).unwrap_or_default() {
        return Some(ps);
    }

    ps.iter_mut().enumerate().for_each(|(i, x)| *x = i as f64);
    if eq.solve(&ps, r).map(|x| x.is_finite()).unwrap_or_default() {
        return Some(ps);
    }

    None
}

fn nfmtr() -> numfmt::Formatter {
    "[~4]".parse::<numfmt::Formatter>().expect("just fine")
}

fn write_rich_table(x: &Fit, write_stats: bool, w: impl Write) -> Result<()> {
    write_table(
        x,
        write_stats,
        comfy_table::presets::UTF8_HORIZONTAL_ONLY,
        w,
    )
    .into_diagnostic()
}

fn write_plain_table(x: &Fit, write_stats: bool, w: impl Write) -> Result<()> {
    write_table(x, write_stats, comfy_table::presets::NOTHING, w).into_diagnostic()
}

fn write_csv_table(x: &Fit, write_stats: bool, mut wtr: impl Write) -> io::Result<()> {
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

    let mut w = csv::Writer::from_writer(&mut wtr);

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
        writeln!(&mut wtr, "  Number of observations: {}", nfmtr.fmt2(*n))?;
        writeln!(
            &mut wtr,
            "  Root Mean Squared Residual error: {}",
            nfmtr.fmt2(*rmsr)
        )?;
        writeln!(&mut wtr, "  R-sq Adjusted: {}", nfmtr.fmt2(*rsq))?;
    }

    Ok(())
}

fn write_md_table(x: &Fit, write_stats: bool, w: impl Write) -> Result<()> {
    write_table(x, write_stats, comfy_table::presets::ASCII_MARKDOWN, w).into_diagnostic()
}

fn write_table(x: &Fit, write_stats: bool, table_fmt: &str, mut w: impl Write) -> io::Result<()> {
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
