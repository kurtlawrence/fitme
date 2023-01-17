use super::*;
use data::Data;
use rmpfit::{MPError, MPFitter, MPResult, MPStatus};

pub struct Fit {
    pub parameter_names: Vec<String>,
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

pub fn fit<E: Equation>(mut eq: E, data: Data, target: &str) -> Result<Fit> {
    let tgt = data
        .headers()
        .find_ignore_case(target)
        .ok_or_else(|| miette!("could not find column '{}' in headers", target))?;

    let fitter = Fitter { data, eq, tgt };

    let mut params = vec![0f64; fitter.eq.params_len()];

    let status = fitter
        .mpfit(&mut params, None, &Default::default())
        .map_err(|e| miette!("{}", e))
        .wrap_err("failed to fit the equation to the input data")?;

    let Fitter { data, mut eq, tgt } = fitter;

    let n = data.len() as f64;
    let k = params.len() as f64;

    let mean_y = data
        .rows()
        .map(|row| row.get(tgt).expect("inside data"))
        .sum::<f64>()
        / n;

    // Y predicition from regression.
    eq.set_params(&params);
    let y_pred: Vec<f64> = data
        .rows()
        .map(|row| eq.solve(row))
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
        .map(|(row, y_)| row.get(tgt).expect("inside data") - y_)
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
        parameter_names: eq.into_params(),
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
        let mut eq = self.eq.clone();
        eq.set_params(params);

        for (d, row) in deviates.iter_mut().zip(self.data.rows()) {
            let f = eq.solve(row).ok_or(MPError::Eval)?;
            let y = row.get(self.tgt).expect("inside data");
            *d = y - f;
        }

        Ok(())
    }
}
