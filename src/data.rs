use super::*;

/// Input data headers representation.
pub struct Headers(Vec<String>);

/// Input data representation.
///
/// Input data is represented as a set of text headers and _rows_ of numbers.
pub struct Data {
    cols: Headers,
    rows: Vec<Vec<Cell>>,
}

pub enum Cell {
    Num(f64),
    Txt(String),
}

macro_rules! cell_impl {
    ($([$t:ty : $v:ident $f:path])*) => {
        $(
            impl From<$t> for Cell {
                fn from(x: $t) -> Cell {
                    Cell::$v($f(x))
                }
            }
        )*
    }
}

cell_impl! {
    [f64:Num std::convert::identity]
    [f32:Num From::from]
    [String:Txt std::convert::identity]
    [&str:Txt ToString::to_string]
}

impl Headers {
    /// The number of headers.
    ///
    /// This is the same as the number of columns.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if there are no headers.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Find the column which matches the string `s`.
    pub fn find(&self, s: &str) -> Option<usize> {
        self.find_match(|x| x.eq(s))
    }

    /// Find the column which matches the string `s`, ignoring ASCII case.
    pub fn find_ignore_case(&self, s: &str) -> Option<usize> {
        self.find_match(|x| x.eq_ignore_ascii_case(s))
    }

    /// Find the column which matches the string `s`, ignoring ASCII case and whitespace.
    pub fn find_ignore_case_and_ws(&self, s: &str) -> Option<usize> {
        self.find_match(|a| str_eq_ignore_case_and_ws(a, s))
    }

    /// Find the column index which matches the predicate.
    pub fn find_match<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(&str) -> bool,
    {
        self.0
            .iter()
            .enumerate()
            .find_map(|(i, x)| predicate(x).then_some(i))
    }

    /// Fuzzily match headers with `s`.
    pub fn fuzzy_match(&self, s: &str) -> impl Iterator<Item = String> + '_ {
        let mut eng = simsearch::SimSearch::new();

        self.0
            .iter()
            .enumerate()
            .for_each(|(i, x)| eng.insert(i, x));

        let r = eng.search(s);

        r.into_iter().filter_map(|i| self.0.get(i)).map(|x| {
            let mut x = x.clone();
            x.retain(|x| !x.is_whitespace());
            x
        })
    }
}

impl<T: AsRef<str>> FromIterator<T> for Headers {
    fn from_iter<I: IntoIterator<Item = T>>(i: I) -> Self {
        Headers(
            i.into_iter()
                .map(|t| t.as_ref().trim().to_string())
                .collect(),
        )
    }
}

impl Data {
    /// Build the input data from the headers and numeric numbers.
    pub fn new<D, R, C>(headers: Headers, data: D) -> Result<Self>
    where
        D: IntoIterator<Item = R>,
        R: IntoIterator<Item = C>,
        C: Into<Cell>,
    {
        let headers_len = headers.len();

        let d = data.into_iter();

        let (l, u) = d.size_hint();
        let mut rows = Vec::with_capacity(u.unwrap_or(l));

        for (i, row) in d.into_iter().enumerate() {
            let row = row.into_iter().map(Into::into).collect::<Vec<_>>();
            ensure!(
                headers_len == row.len(),
                "row index {} does not have the same length as the headers",
                i
            );
            rows.push(row);
        }

        Ok(Self {
            cols: headers,
            rows,
        })
    }

    /// Returns the length of the number of observation rows.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Returns if there is no **data rows**. (There may still be headers).
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// The set of headers.
    pub fn headers(&self) -> &Headers {
        &self.cols
    }

    /// Return an iterator of [`DataRow`].
    pub fn rows(&self) -> impl ExactSizeIterator<Item = DataRow> {
        self.rows.iter().enumerate().map(|(idx, vals)| DataRow {
            idx,
            vals,
            hdrs: &self.cols,
        })
    }
}

/// A single row [`Data`].
#[derive(Copy, Clone)]
pub struct DataRow<'a> {
    idx: usize,
    vals: &'a [Cell],
    hdrs: &'a Headers,
}

impl<'a> DataRow<'a> {
    /// Get the number value at the column index.
    ///
    /// If the cell is not a number, a location error is returned.
    pub fn get_num(&self, colidx: usize) -> Option<Result<f64>> {
        self.vals.get(colidx).map(|c| match c {
            Cell::Num(x) => Ok(*x),
            Cell::Txt(x) => Err(miette!("failed to parse '{}' as number", x))
                .wrap_err_with(|| format!("in column index {colidx}"))
                .wrap_err_with(|| format!("in row index {}", self.idx + 1)),
        })
    }

    /// The row index.
    pub fn idx(&self) -> usize {
        self.idx
    }

    /// The [`Data`] headers.
    pub fn headers(&self) -> &Headers {
        self.hdrs
    }
}

pub struct CsvReader {
    rdr: csv::Reader<Box<dyn std::io::Read>>,
    cols: Option<Headers>,
}

impl CsvReader {
    pub fn new<R: std::io::Read + 'static>(rdr: R) -> Self {
        Self {
            rdr: csv::Reader::from_reader(Box::new(rdr)),
            cols: None,
        }
    }

    fn read_headers(&mut self) -> Result<()> {
        let hdrs = self
            .rdr
            .headers()
            .into_diagnostic()
            .wrap_err("failed to read CSV header row")?;

        ensure!(!hdrs.is_empty(), "headers row is empty");

        self.cols = Some(hdrs.iter().collect());
        Ok(())
    }

    pub fn headers(&mut self) -> Result<&Headers> {
        if self.cols.is_none() {
            self.read_headers()?;
        }

        self.cols
            .as_ref()
            .map(Ok)
            .expect("should be some if read_headers succeeds")
    }

    pub fn into_data(self) -> Result<Data> {
        Data::try_from(self)
    }
}

impl TryFrom<CsvReader> for Data {
    type Error = miette::Report;

    fn try_from(mut rdr: CsvReader) -> Result<Data> {
        rdr.headers()?; // ensure headers is read in

        let mut data = Vec::new();

        for (i, row) in rdr.rdr.records().enumerate() {
            let row = row
                .into_diagnostic()
                .wrap_err_with(|| format!("failed to read row {} in CSV", i + 1))?;

            let row: Vec<Cell> = row
                .iter()
                .map(|cell| {
                    cell.parse::<f64>()
                        .map(Cell::Num)
                        .unwrap_or_else(|_| Cell::Txt(cell.to_string()))
                })
                .collect();

            data.push(row);
        }

        let headers = rdr.cols.expect("headers should be initialised");

        Data::new(headers, data)
    }
}

fn str_eq_ignore_case_and_ws(a: &str, b: &str) -> bool {
    let mut a = a.chars().filter(|x| !x.is_whitespace());
    let mut b = b.chars().filter(|x| !x.is_whitespace());

    loop {
        match (a.next(), b.next()) {
            (None, None) => break true,
            (None, Some(_)) | (Some(_), None) => break false, // not the same length
            (Some(a), Some(b)) => match a.eq_ignore_ascii_case(&b) {
                true => (),           // check next character
                false => break false, // chars do not match
            },
        }
    }
}

pub(crate) fn match_hdr_help(hdrs: &Headers, col: &str) -> String {
    let mut s = String::from("help - these headers are similar:");
    let l = s.len();

    for h in hdrs.fuzzy_match(col) {
        s.push(' ');
        s.push_str(&h);
    }

    if s.len() == l {
        s.clear();
        s.push_str("help - no columns match, use `cat <file> | head -n1` for inspect headers");
    }

    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use fastrand::*;
    use std::iter::*;

    #[test]
    fn eq_testing() {
        use str_eq_ignore_case_and_ws as f;
        assert!(f("", ""));
        assert!(f("  ", " "));
        assert!(f("  a  ", " a"));
    }

    #[test]
    fn fuzz_eq_testing() {
        for _ in 0..10_000 {
            let a: String = repeat_with(|| char(..)).take(usize(..100)).collect();
            let b = a.chars().fold(String::new(), |mut s, c| {
                s.extend(repeat(' ').take(usize(..2)));
                s.push(c);
                s
            });

            assert!(str_eq_ignore_case_and_ws(&a, &b));
        }
    }

    #[test]
    fn fuzz_eq_testing2() {
        for _ in 0..10_000 {
            let mut a: String = repeat_with(|| char(..)).take(usize(..100)).collect();
            let mut b: String = repeat_with(|| char(..)).take(usize(..100)).collect();

            let x = str_eq_ignore_case_and_ws(&a, &b);

            a.retain(|c| !c.is_whitespace());
            b.retain(|c| !c.is_whitespace());

            let y = a.eq_ignore_ascii_case(&b);

            assert_eq!(x, y);
        }
    }
}
