extern crate num_cpus;

extern crate indicatif;
use indicatif::{MultiProgress, ProgressBar, HumanBytes};

extern crate rand;
use rand::{thread_rng, Rng};

extern crate threadpool;
use threadpool::ThreadPool;

extern crate clap;
use clap::{Arg, App};

use std::ffi::{OsString, OsStr};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

struct DataBuffer {
    data: Vec<u8>,
    data_type: DataType,
}

impl DataBuffer {
    fn new(data_type: DataType, len: usize) -> Self {
        DataBuffer {
            data: vec![0; len],
            data_type: data_type,
        }
    }

    fn next_bytes(&mut self) -> &[u8] {
        match self.data_type {
            DataType::Random => {
                let mut rng = thread_rng();
                rng.fill_bytes(&mut self.data); // Maybe use gen() instead?
                &self.data
            },
            DataType::Zeroes => {
                &self.data
            }
        }
    }
}

#[derive(Copy, Clone)]
enum DataType {
    Random,
    Zeroes,
}

fn is_valid_int(s: &OsStr) -> Result<(), OsString> {
    match s.to_string_lossy().parse::<usize>() {
        Ok(_) => Ok(()),
        Err(e) => Err(OsString::from(e.to_string())),
    }
}

fn main() {
    let matches = App::new("diskdestroyer")
        .version(env!("CARGO_PKG_VERSION"))
        .arg(Arg::with_name("zero")
             .short("z")
             .long("zero")
             .validator_os(is_valid_int)
             .help("Write zeroes instead of random data"))
        .arg(Arg::with_name("threads")
             .short("t")
             .long("threads")
             .value_name("THREADS")
             .validator_os(is_valid_int)
             .help("Set number of threads to write with (Default: # of CPU cores)"))
        .arg(Arg::with_name("blocksize")
             .short("b")
             .long("blocksize")
             .value_name("BYTES")
             .help("Specify the number of bytes written at a time (Default: 1024)"))
        .arg(Arg::with_name("files")
             .index(1)
             .required(true)
             .help("The file(s) to write data to")
             .value_name("FILES")
             .multiple(true))
        .get_matches();

    let bs: usize = match matches.value_of_os("blocksize") {
        Some(val) => val.to_string_lossy().parse::<usize>().unwrap(),
        None => 1024,
    };

    let datatype = match matches.is_present("zero") {
        false => DataType::Random,
        true => DataType::Zeroes,
    };

    let threads: usize = match matches.value_of_os("threads") {
        Some(val) => val.to_string_lossy().parse::<usize>().unwrap(),
        None => num_cpus::get(),
    };

    let paths: Vec<PathBuf> = matches.values_of_os("files").unwrap().map(|f| PathBuf::from(f)).collect();

    let pool = ThreadPool::new(threads);

    let progress = MultiProgress::new();

    for path in paths {
        let spinner = progress.add(ProgressBar::new_spinner());
        pool.execute(move || {
            let mut buffer: DataBuffer = DataBuffer::new(datatype, bs);

            match OpenOptions::new().write(true).open(&path) {
                Ok(mut file) => {
                    let mut written = 0;
                    while let Ok(bytes) = file.write(buffer.next_bytes()) {
                        written += bytes;
                        spinner.set_message(&format!("{}: {}", &path.to_string_lossy(), HumanBytes(written as u64)));
                        spinner.tick();
                    }
                    spinner.finish_with_message(&format!("{}: complete", &path.to_string_lossy()));
                },
                Err(e) => {
                    eprintln!("Failed to open {}: {}", &path.to_string_lossy(), e);
                }
            }
        });
    }

    let _ = progress.join();
    pool.join();
}
