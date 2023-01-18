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
