use std::fs::File;
use std::io::{BufRead, BufReader, Error};
use std::path::Path;

pub struct Csv {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

impl Csv {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let mut columns: Vec<String> = vec![];
        let mut rows: Vec<Vec<String>> = vec![];
        let file = File::open(path)?;
        let file = BufReader::new(file);
        for (i, line) in file.lines().enumerate() {
            let line = line?;
            if i == 0 {
                columns = line.split(',').map(|x| x.to_string()).collect();
                continue;
            }
            let line: Vec<String> = line
                .replace(",,", ", ,")
                .split(',')
                .map(|x| x.to_string())
                .collect();
            if line.len() != columns.len() {
                panic!("length mismatch in csv file, check row {}", i + 1);
            }
            rows.push(line);
        }
        Ok(Self { columns, rows })
    }
}
