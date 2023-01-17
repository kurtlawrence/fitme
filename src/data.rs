use super::*;

pub struct Headers(Vec<String>);

pub struct Data {
    cols: Headers,
    rows: Vec<Vec<f64>>,
}

impl Headers {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn find(&self, s: &str) -> Option<usize> {
        self.find_match(|x| x.eq(s))
    }

    pub fn find_ignore_case(&self, s: &str) -> Option<usize> {
        self.find_match(|x| x.eq_ignore_ascii_case(s))
    }

    pub fn find_match<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(&str) -> bool,
    {
        self.0
            .iter()
            .enumerate()
            .find_map(|(i, x)| predicate(x).then_some(i))
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
    pub fn new(headers: Headers, data: Vec<Vec<f64>>) -> Result<Self> {
        let headers_len = headers.len();

        for (i, row) in data.iter().enumerate() {
            ensure!(
                headers_len == row.len(),
                "row index {} does not have the same length as the headers",
                i
            );
        }

        Ok(Self {
            cols: headers,
            rows: data,
        })
    }

    /// Returns the length of the number of observation rows.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn headers(&self) -> &Headers {
        &self.cols
    }

    pub fn rows(&self) -> impl ExactSizeIterator<Item = DataRow> {
        self.rows.iter().enumerate().map(|(idx, vals)| DataRow {
            idx,
            vals,
            hdrs: &self.cols,
        })
    }
}

#[derive(Copy, Clone)]
pub struct DataRow<'a> {
    idx: usize,
    vals: &'a [f64],
    hdrs: &'a Headers,
}

impl<'a> DataRow<'a> {
    pub fn get(&self, colidx: usize) -> Option<f64> {
        self.vals.get(colidx).copied()
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

            let row: Vec<f64> = row
                .iter()
                .enumerate()
                .map(|(j, cell)| {
                    cell.parse::<f64>()
                        .into_diagnostic()
                        .wrap_err_with(|| format!("in column index {j}"))
                        .wrap_err_with(|| format!("in row index {}", i + 1))
                })
                .try_fold(Vec::new(), |mut v, f| {
                    f.map(|f| {
                        v.push(f);
                        v
                    })
                })?;

            data.push(row);
        }

        let headers = rdr.cols.expect("headers should be initialised");

        Data::new(headers, data)
    }
}
