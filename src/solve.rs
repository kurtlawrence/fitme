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

struct Fitter<E> {
    data: Data,
    eq: E,
    tgt: usize,
}

/// Fit an equation using the input data.
///
/// If you are using `fitme` as a library, this is function to use!
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
/// assert_eq!(&fit.parameter_names, &["m".to_string(), "c".to_string()]);
/// assert_eq!(&fit.parameter_values, &[1.7709542026136489, 3.209965716831507]);
/// ```
pub fn fit<E: Equation>(eq: E, data: Data, target: &str) -> Result<Fit> {
    let tgt = data
        .headers()
        .find_ignore_case_and_ws(target)
        .ok_or_else(|| miette!("could not find column '{}' in headers", target))
        .wrap_err_with(|| data::match_hdr_help(data.headers(), target))?;

    ensure_float_values_in_data(&eq, &data, tgt)?;

    let fitter = Fitter { data, eq, tgt };

    let mut params = vec![1e-3; fitter.eq.params_len()];

    if params.is_empty() {
        let mut x = Err(miette!("equation has 0 parameters to fit")).wrap_err(
            "equation must have a least one variable which does not match a column header",
        );
        if let Some(e) = fitter.eq.expr() {
            x = x.wrap_err_with(|| format!("supplied expr: {e}"));
        }

        return x;
    }

    let status = fitter
        .mpfit(&mut params, None, &Default::default())
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
