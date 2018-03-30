extern crate num_cpus;

extern crate rand;
use rand::{StdRng, Rng};

extern crate threadpool;
use threadpool::ThreadPool;

extern crate clap;
use clap::{Arg, App};

use std::ffi::{OsString, OsStr};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{PathBuf, Path};
use std::process::exit;

struct DataBuffer {
    data: Vec<u8>,
    rng: Option<StdRng>,
    data_type: DataType,
}

impl DataBuffer {
    fn new(data_type: DataType, len: usize) -> Result<Self, String> {
        match data_type {
            DataType::Random => {
                let rng = match StdRng::new() {
                    Ok(rng) => rng,
                    Err(e) => {
                        return Err(format!("Can't create random number generator: {}", e));
                    },
                };

                Ok(
                    DataBuffer {
                        data: vec![0; len],
                        rng: Some(rng),
                        data_type: data_type,
                    }
                )
            },
            DataType::Zeroes => {
                Ok(
                    DataBuffer {
                        data: vec![0; len],
                        rng: None,
                        data_type: data_type,
                    }
                )
            },
        }
    }

    fn next_bytes(&mut self) -> &[u8] {
        match self.data_type {
            DataType::Random => {
                self.rng.unwrap().fill_bytes(&mut self.data); // Maybe use gen() instead?
                &self.data
            },
            DataType::Zeroes => {
                &self.data
            }
        }
    }
}

enum DataType {
    Random,
    Zeroes,
}

fn is_valid_int(s: &OsStr) -> Result<(), OsString> {
    match s.to_string_lossy().parse::<usize>() {
        Ok(int) => Ok(()),
        Err(e) => Err(OsString::from(e.to_string())),
    }
}

fn write_data(path: &Path, data: &[u8]) {

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

    let mut buffer: DataBuffer = unimplemented!();

    for path in paths {
        pool.execute(move || {
            match OpenOptions::new().write(true).open(&path) {
                Ok(mut file) => {
                    println!("Starting to write to {}", &path.to_string_lossy());
                    while let Ok(_bytes) = file.write(buffer.next_bytes()) {

                    }
                },
                Err(e) => {
                    eprintln!("Failed to open {}: {}", &path.to_string_lossy(), e);
                }
            }
        });
    }

    pool.join();
}
