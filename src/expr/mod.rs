//! Mathematical expression parsing and evaluation.

use super::*;
use data::{DataRow, Headers};

pub mod v1;

/// Parse and solve a mathematical expression.
pub trait Equation: Sized {
    /// Parse a text expression into an expression.
    fn parse(expr: &str, columns: &Headers) -> Result<Self>;

    /// The number of free parameters.
    fn params_len(&self) -> usize;

    /// Evaluate the expression with the given set of parameters and a single data row.
    fn solve(&self, params: &[f64], row: DataRow) -> Option<f64>;

    /// Fetch the string form of the expression, if it exists.
    fn expr(&self) -> Option<String>;

    /// Extract out the parameter names.
    fn params(&self) -> Vec<String>;

    /// Extract out the variable names.
    fn vars(&self) -> Vec<String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distinct_params_and_vars() {
        fn test<E: Equation>() {
            let e = E::parse("x * x + d + d", &Headers::from_iter(["d"])).unwrap();

            assert_eq!(e.params(), vec!["x".to_string()]);
            assert_eq!(e.vars(), vec!["d".to_string()]);
        }

        test::<v1::Eq>();
    }
}
