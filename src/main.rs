#[macro_use]
extern crate chan;
extern crate chan_signal;
extern crate chrono;
#[macro_use]
extern crate failure;

use std::env;
use std::fs;
use std::io;
use std::io::BufRead;
use std::io::Write;

use chan_signal::Signal;
use failure::Error;
use failure::ResultExt;

fn main() -> Result<(), Error> {
    let signal = {
        let (s, r) = chan::async();
        chan_signal::notify_on(&s, Signal::USR1);
        r
    };

    let prefix = env::args().nth(1).unwrap_or("rotaters-".to_string());

    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    let mut line = Vec::with_capacity(1024);

    let mut writing = new_file(&prefix)?;

    loop {
        line.clear();
        if 0 == stdin.read_until(b'\n', &mut line)? {
            break Ok(());
        }

        let select;
        chan_select!(default => { select = false; }, signal.recv() => { select = true; },);

        if select {
            writing = new_file(&prefix)?;
        }

        stdout.write_all(&line)?;
        writing.write_all(&line)?;
    }
}

fn new_file(prefix: &str) -> Result<io::BufWriter<fs::File>, Error> {
    let path_candidate = format!("{}-{}", prefix, chrono::Utc::now());
    Ok(io::BufWriter::new(
        fs::File::create(&path_candidate)
            .with_context(|_| format_err!("creating {}", path_candidate))?,
    ))
}
