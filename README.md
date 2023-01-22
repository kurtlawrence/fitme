# fitme
CLI curve fitting tool. Parameterise an equation from a CSV dataset.

> `fitme` is primarily a CLI tool, and this README details the CLI use.
>
> If one is wanting to use `fitme` as a library, please see the [API docs](https://docs.rs/fitme).

---

```plaintext
> fitme y "m * x + c" tests/file1.csv
──────────────────────────────────────────────
 Parameter   Value   Standard Error   t-value 
══════════════════════════════════════════════
 c           3.209            0.013     230.3 
──────────────────────────────────────────────
 m           1.770            0.011     149.0 
──────────────────────────────────────────────
  Number of observations: 10.0
  Root Mean Squared Residual error: 0.043
  R-sq Adjusted: 0.999
```

# Features

- simple interface
- fast
- flexible equations
- helpful error messages

# Installation

Currently, only installation from source is supported:

```plaintext
# using crates.io
cargo install fitme

# using github
cargo install --git https://github.com/kdr-aus/fitme
```

# Usage

- `fitme --help` for detailed help.

`fitme` requires just two arguments, the target column to fit against, and the mathematical
expression. The third optional argument specifies the file to read the CSV from.
`fitme` uses a least-squares fitting approach.

Let's fit a linear regression to the following data:

|  y  |  x  |
| --- | --- |
| 1.9000429E-01 | -1.7237128E+00 |
| 6.5807428E+00 | 1.8712276E+00 |
| 1.4582725E+00 | -9.6608055E-01 |
| 2.7270851E+00 | -2.8394297E-01 |
| 5.5969253E+00 | 1.3416969E+00 |
| 5.6249280E+00 | 1.3757038E+00 |
| 0.787615 | -1.3703436E+00 |
| 3.2599759E+00 | 4.2581975E-02 |
| 2.9771762E+00 | -1.4970151E-01 |
| 4.5936475E+00 | 8.2065094E-01 |

Equation: `y = m * x + c`

Here the:
- _target_: `y`
- _variables_: `x`
- _parameters_: `m`, `c`

To run a fit, simply use `fitme y "m * x + c" test-file.csv`:

```plaintext
> fitme y "m * x + c" test-file.csv
──────────────────────────────────────────────
 Parameter   Value   Standard Error   t-value 
══════════════════════════════════════════════
 c           3.209            0.013     230.3 
──────────────────────────────────────────────
 m           1.770            0.011     149.0 
──────────────────────────────────────────────
  Number of observations: 10.0
  Root Mean Squared Residual error: 0.043
  R-sq Adjusted: 0.999
```

Notice that `fitme` will automatically match column names in the equation, binding them as
**variables**. Unmatched variables become **parameters**.

## Multi Parameters

`fitme` is useful for fitting multiple least squares linear regressions:

```plaintext
> fitme sepalLength "a * petalLength + b * sepalWidth + c * petalWidth + d" iris.csv
───────────────────────────────────────────────
 Parameter   Value    Standard Error   t-value 
═══════════════════════════════════════════════
 a            0.711            0.056     12.51 
───────────────────────────────────────────────
 b            0.654            0.066     9.788 
───────────────────────────────────────────────
 c           -0.562            0.127    -4.410 
───────────────────────────────────────────────
 d            1.845            0.251     7.342 
───────────────────────────────────────────────
  Number of observations: 150.0
  Root Mean Squared Residual error: 0.314
  R-sq Adjusted: 0.855
```





## Flexible Output

Alter the output via the `--out` switch.

### CSV
```plaintext
> fitme y "m * x + c" file1.csv -o=csv -n
Parameter,Value,Standard Error,t-value
m,1.7709542029456211,0.011883297834310212,149.02884936809457
c,3.2099657167997013,0.013936863525869892,230.32195951702457
```

### Markdown
```plaintext
> fitme y "m * x + c" file1.csv -o=md -n
| Parameter | Value | Standard Error | t-value |
|-----------|-------|----------------|---------|
| c         | 3.209 |          0.013 |   230.3 |
| m         | 1.770 |          0.011 |   149.0 |
```

### JSON
```plaintext
> fitme y "m * x + c" file1.csv -o=json -n
{"parameter_names":["m","c"],"parameter_values":[1.7709542029456211,3.2099657167997013],"n":10,"xerrs":[0.011883297834310212,0.013936863525869892],"rmsr":0.04392493014188053,"rsq":0.9995948974725735,"tvals":[149.02884936809457,230.32195951702457]}
```

### + more!


# Mathematical Expressions

- `+,-,*,/`
- `%`: remainder
- `^`: power
- `pi, e`
- `sqrt(), abs()`
- `exp(), ln(), log()`
- `sin(), cos(), tan()`
- `sinh(), cosh(), tanh()`
- `floor(), ceil(), round()`

🔬 If you need more math support, please [raise an issue](https://github.com/kdr-aus/fitme/issues).

# Example Equations

## Linear

- Equation: `y = Ax + B`
- Columns: `y, x`
- Parameters: `A, B`

```bash
> fitme y "Ax + B"
```

## Multiple Linear Regression

- Equation: `y = P0 * x0 + P1 * x1 + ... + Pn * xn + C`
- Columns: `y, x0, x1, ... , xn`
- Parameters: `P0, P1, ... , Pn, C`

```bash
> fitme y "P0 * x0 + P1 * x1 + ... + Pn * xn + C"
```

## Normal Distribution

The goal is to fit to a CDF, so the input CSV will have _P_ as the probability [0,1], and 
_x_ as the variable.

$$P = {1\over2} \bigg\lbrack {1 + erf \Big( {{x-\mu}\over{\sigma^2\sqrt2}} \Big)}\bigg\rbrack$$

We can [approximate the `erf` function with](https://math.stackexchange.com/questions/321569/approximating-the-error-function-erf-by-analytical-functions):

$$erf(x) \approx \tanh \big( {\sqrt{\pi}\log(2)x} \big)$$

So:

```math
P = {1\over2} \bigg\lbrack 
  {1 + \tanh \Big( 
    {{(x-\mu)\sqrt\pi\log(2)}\over{\sigma^2\sqrt2}} 
  \Big)}
\bigg\rbrack
```

This transforms into the expression:
```plaintext
0.5 * (1 + tanh(((x - Mean) * sqrt(pi) * log(2)) / (Stdev^2 * sqrt(2))))

Parameters: Mean, Stdev
Variables: x
```

And to fit:

```bash
> fitme P  "0.5 * (1 + tanh(((x - Mean) * sqrt(pi) * log(2)) / (Stdev^2 * sqrt(2))))"
```