mod csv;
use std::error::Error as ET;
use std::fs::File;
use std::io::{BufRead, BufReader, Error};
use std::path::Path;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, Mutex};
use std::thread;

use csv::*;

mod email;
use email::*;

mod config;
use config::*;

fn handle_template<P: AsRef<Path>>(path: P, row: usize, data: &Csv) -> Result<String, Error> {
    let file = File::open(path)?;
    let file = BufReader::new(file);
    let mut final_msg = String::new();
    for line in file.lines() {
        let line = line?;
        let mut open = false;
        let mut term = String::new();
        let mut new_line = String::new();
        for c in line.chars() {
            match c {
                '{' => open = true,
                '}' => {
                    open = false;
                    for (i, col) in data.columns.iter().enumerate() {
                        if col.to_lowercase() != term.to_lowercase() {
                            continue;
                        }
                        new_line.push_str(&data.rows[row][i]);
                    }
                    term.clear();
                }
                _ => {
                    if open && c != '{' {
                        term.push(c);
                        continue;
                    }
                    new_line.push(c);
                }
            }
        }
        final_msg.push_str(&(new_line + "\n"));
    }
    Ok(final_msg)
}

static CORES: usize = 4;

fn main() -> Result<(), Box<dyn ET>> {
    let display: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let data = Arc::new(Csv::from_file("./data.csv")?);
    let per_core = Arc::new(data.rows.len() / CORES);
    let total_sent = Arc::new(AtomicUsize::new(1));
    let config = Arc::new(Config::load());
    let email_index = {
        let mut index = 0;
        for (i, v) in data.columns.iter().enumerate() {
            if v.to_lowercase().as_str() == "email" {
                index = i;
                break;
            }
            panic!("no email column detected");
        }
        index
    };
    for i in 0..CORES {
        let i = Arc::new(i);
        let i = i.clone();
        let display = display.clone();
        let config = config.clone();
        let data = data.clone();
        let per_core = per_core.clone();
        let total_sent = total_sent.clone();
        thread::spawn(move || {
            let ending = if *i == CORES - 1 {
                data.rows.len() - 1
            } else {
                (*i + 1) * *per_core
            };
            let mut sent = 0;
            for r in *i * *per_core..ending {
                let template = handle_template("template.txt", r, &data);
                if let Ok(v) = template {
                    if email(
                        &v,
                        &config.username,
                        &data.rows[r][email_index],
                        &config.subject,
                        &config,
                    )
                    .is_ok()
                    {
                        sent += 1;
                        display.lock().unwrap()[*i] = format!(
                            "[CORE {}] sent {}/{}",
                            *i + 1,
                            sent,
                            ending - *i * *per_core
                        );
                        let total = total_sent.load(Relaxed);
                        total_sent.swap(total + 1, Relaxed);
                    } else {
                        display.lock().unwrap()[*i] = format!(
                            "[CORE {}] sent {}/{} - FAILURE",
                            *i + 1,
                            sent,
                            ending - *i * *per_core
                        );
                    }
                }
            }
        });
    }
    loop {
        let email_total = total_sent.load(Relaxed);
        if email_total >= data.rows.len() {
            break;
        }
        let mut output = String::new();
        for i in &*display.lock().unwrap() {
            output.push_str(i);
        }
        output.push_str(&format!("TOTAL: {email_total}/{}", data.rows.len()));
        println!("\x1b[{CORES}A{output}");
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::Write;

    use crate::csv::Csv;
    use crate::handle_template;

    macro_rules! nvec {
        ($v:expr) => {
            $v.iter().map(|x| x.to_string()).collect()
        };
    }

    #[test]
    fn gen_template_1() {
        let csv = Csv {
            columns: nvec!(vec!["email", "name", "greeting", "ending"]),
            rows: vec![
                nvec!(vec!["test@gmail.com", "test", "hello", "bye"]),
                nvec!(vec!["bob@gmail.com", "bob", "good morning", "cya"]),
                nvec!(vec!["jeff@gmail.com", "jeff", "fr", "sorry"]),
                nvec!(vec!["crack@gmail.com", "crack", "no cap", "fr"]),
                nvec!(vec!["noah@gmail.com", "noah", "ong", "bye"]),
                nvec!(vec!["john@gmail.com", "john", "hello", "yup"]),
            ],
        };
        let f = File::create("template.txt");
        if let Ok(mut v) = f {
            writeln!(
                &mut v,
                r#"
To {{email}}
{{greeting}} {{name}}, I hope you are having a great day.

Best regards {{ending}},
{{name}}
"#
            )
            .expect("failed to write to template file");
        }
        for i in 0..6 {
            let res = handle_template("template.txt", i, &csv);
            assert_eq!(
                res.unwrap().trim(),
                format!(
                    "\n\
To {}\n\
{} {}, I hope you are having a great day.\n\
\n\
Best regards {},\n\
{}\n\
",
                    csv.rows[i][0], csv.rows[i][2], csv.rows[i][1], csv.rows[i][3], csv.rows[i][1]
                )
                .trim()
            );
        }
    }
}
