//! Version 1 of the equation resolver.

use super::*;
use meval::{tokenizer::Token, Expr};

/*** A note on the implementation ***
 *
 * `meval` parses an expression to obtain a token stream, however when binding, the stream is
 * consumed. Binding also requires a lifetime of the variables for the function to live, this is
 * unfortunately not available with the `Equation` trait without changing the `parse` function,
 * which I am fairly unwilling to do.
 *
 * Instead, the bind happens on the `solve` method, both params and variables are bound in this
 * call which solves the lifetime issue.
 */

/// Version 1 of the equation resolver.
///
/// Equations are expected to be the typical RHS. For instance, to solve for `y = m * x + c`, the
/// equation to parse is `m * x + c`.
#[derive(Clone)]
pub struct Eq {
    /// Variable (column) bindings.
    vars: Vec<(String, usize)>,

    /// Unmapped variables represent the parameters to twiddle with.
    params: Vec<String>,

    /// Parsed expression.
    expr: Expr,
}

impl Equation for Eq {
    fn parse(expr: &str, columns: &Headers) -> Result<Self> {
        let func = expr
            .parse::<Expr>()
            .into_diagnostic()
            .wrap_err_with(|| format!("parsing '{expr}' failed"))?;

        // map any *matched* variables as column variables, and
        // any *unmatched* variables as parameters
        let mut vars = Vec::new();
        let mut params = Vec::new();
        for t in func.iter() {
            if let Token::Var(n) = t {
                match columns.find_ignore_case(n) {
                    Some(i) => vars.push((n.to_string(), i)),
                    None => params.push(n.to_string()),
                }
            }
        }

        Ok(Self {
            vars,
            params,
            expr: func,
        })
    }

    fn params_len(&self) -> usize {
        self.params.len()
    }

    fn solve(&self, params: &[f64], row: DataRow) -> Option<f64> {
        // build a vector of the params + variable names
        let vars = self
            .params
            .iter()
            .map(String::as_str)
            .chain(self.vars.iter().map(|(x, _)| x.as_str()))
            .collect::<Vec<_>>();

        // bind the expression to the variables
        let f = self.expr.clone().bindn(&vars).ok()?;

        // build the inputs
        let mut inputs = Vec::with_capacity(vars.len());

        inputs.extend_from_slice(params); // first, the stored params
        for (_, i) in &self.vars {
            inputs.push(row.get(*i)?); // then the params
        }

        Some(f(&inputs)) // eval the function
    }

    fn into_params(self) -> Vec<String> {
        self.params
    }
}