use super::*;
use data::{DataRow, Headers};

pub mod v1;

pub trait Equation: Sized + Clone {
    fn parse(expr: &str, columns: &Headers) -> Result<Self>;

    fn params_len(&self) -> usize;

    fn set_params(&mut self, params: &[f64]);

    fn solve(&self, row: DataRow) -> Option<f64>;

    fn into_params(self) -> Vec<String>;
}
