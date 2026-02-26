use std::fmt::Display;

use clap::Parser;

mod cli;
mod bzip2;

type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    Array(Vec<Error>),
    Io(std::io::Error),
    CannotWriteToStdout(),
    CannotGuessOriginalName(String),
    FileExists(String),
    InvalidInput(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Array(errs) => {
                for err in errs {
                    writeln!(f, "{err}")?;
                }
                Ok(())
            },
            Error::Io(e) => write!(f, "I/O error: {e}"),
            Error::FileExists(file) => write!(f, "bzip2: Output file {file} already exists."),
            Error::InvalidInput(msg) => write!(f, "{msg}"),
            Error::CannotWriteToStdout() => write!(f, "bzip2: I won't write compressed data to a terminal. Use -c for redirecting the output to a file."),
            Error::CannotGuessOriginalName(name) => write!(f, "bzip2: Can't guess original name for {name} -- using {name}.out instead")
        }
    }
}

impl Error {
    fn error_or<T>(ok_item: T, errs: Vec<Error>) -> Result<T> {
        if errs.is_empty() {
            Ok(ok_item)
        } else if errs.len() == 1 {
            Err(errs.into_iter().next().unwrap())
        } else {
            Err(Error::Array(errs))
        }
    }
}

fn decompress_each(file: &str, dest: &str, errs: &mut Vec<Error>, cli: &cli::Bzip2Cli, program_name: &str) {
    match std::fs::File::open(file) {
        Ok(input_file) => {
            if cli.is_stdout(program_name) {
                match bzip2::decompress(input_file, std::io::stdout(), cli) {
                    Ok(bytes) => log::info!("{file}: Decompressed to stdout ({bytes} bytes)"),
                    Err(e) => errs.push(e),
                }
            } else {
                match std::fs::File::create(dest) {
                    Ok(output_file) => match bzip2::decompress(input_file, output_file, cli) {
                        Ok(bytes) => log::info!("{file}: Decompressed to {dest} ({bytes} bytes)"),
                        Err(e) => errs.push(e),
                    }
                    Err(e) => errs.push(Error::Io(e)),
                }
            }
        },
        Err(e) => errs.push(Error::Io(e)),
    }
    if !cli.keep {
        match std::fs::remove_file(file) {
            Ok(_) => log::info!("{file}: Deleted original file"),
            Err(e) => errs.push(Error::Io(e)),
        }
    }
}

fn perform_decompress(cli: &cli::Bzip2Cli, program_name: &str) -> Result<()> {
    log::info!("Decompressing files...");
    let mut errs = vec![];
    for file in cli.iter() {
        log::info!("{file}: Decompressing file");
        let dest = if !file.ends_with(".bz2") {
            errs.push(Error::CannotGuessOriginalName(file.clone()));
            continue;
        } else {
            file.strip_suffix(".bz2").unwrap()
        };
        if !cli.force && std::path::Path::new(dest).exists() {
            errs.push(Error::FileExists(dest.to_string()));
            continue;
        }
        decompress_each(file, dest, &mut errs, cli, program_name);
    }
    if cli.is_empty() {
        if cli.is_stdout(program_name) {
            match bzip2::decompress(std::io::stdin(), std::io::stdout(), cli) {
                Ok(bytes) => log::info!("stdin: Decompressed to stdout ({bytes} bytes)"),
                Err(e) => errs.push(e),
            }
        } else {
            errs.push(Error::CannotWriteToStdout())
        }
    }

    Error::error_or((), errs)
}

fn compress(file: &str, dest: &str, errs: &mut Vec<Error>, cli: &cli::Bzip2Cli) {
    match std::fs::File::open(file) {
        Ok(input_file) => {
            if cli.stdout {
                match bzip2::compress(input_file, std::io::stdout(), cli) {
                    Ok(bytes) => log::info!("{file}: Compressed to stdout ({bytes} bytes)"),
                    Err(e) => errs.push(e),
                }
            } else {
                match std::fs::File::create(dest) {
                    Ok(output_file) => match bzip2::compress(input_file, output_file, cli) {
                        Ok(bytes) => log::info!("{file}: Compressed to {dest} ({bytes} bytes)"),
                        Err(e) => errs.push(e),
                    }
                    Err(e) => errs.push(Error::Io(e)),
                }
            }
        },
        Err(e) => errs.push(Error::Io(e)),
    }
    if !cli.keep {
        match std::fs::remove_file(file) {
            Ok(_) => log::info!("{file}: Deleted original file"),
            Err(e) => errs.push(Error::Io(e)),
        }
    }
}

fn perform_compress(cli: &cli::Bzip2Cli) -> Result<()> {
    log::info!("Compressing files...");
    let mut errs = vec![];
    for file in cli.iter() {
        if file.ends_with(".bz2") {
            errs.push(Error::InvalidInput(format!("bzip2: Input file {file} already has .bz2 suffix.")));
            continue;
        }
        log::info!("{file}: Compressing file");
        let dest = format!("{file}.bz2");
        if !cli.force && std::path::Path::new(&dest).exists() {
            errs.push(Error::FileExists(dest));
            continue;
        }
        compress(file, &dest, &mut errs, cli);
    }
    if cli.is_empty() {
        if cli.stdout {
            match bzip2::compress(std::io::stdin(), std::io::stdout(), cli) {
                Ok(bytes) => log::info!("stdin: Compressed to stdout ({bytes} bytes)"),
                Err(e) => errs.push(e),
            }
        } else {
            errs.push(Error::CannotWriteToStdout())
        }
    }
    Error::error_or((), errs)
}

fn perform_test(cli: &cli::Bzip2Cli) -> Result<()> {
    log::info!("Testing integrity of compressed files...");
    let mut errs = vec![];
    for file in cli.iter() {
        log::info!("{file}: Testing file");
        match std::fs::File::open(file) {
            Ok(f) => {
                match bzip2::test_integrity(f) {
                    Ok(bytes) => log::info!("{file}: OK ({bytes} bytes)"),
                    Err(e) => errs.push(e),
                }
            },
            Err(e) => errs.push(Error::Io(e)),
        };
    }
    Error::error_or((), errs)
}

pub(crate) fn do_main<S: AsRef<str>>(args: Vec<S>) -> Result<()> {
    let args = args.into_iter().map(|s| s.as_ref().to_string()).collect::<Vec<String>>();
    let pname = args[0].split('/').next_back().unwrap_or(&args[0]);
    let cli = cli::Bzip2Cli::parse_from(&args);
    if !cli.init() {
        match cli.mode(pname) {
            cli::Mode::Compress => perform_compress(&cli),
            cli::Mode::Decompress => perform_decompress(&cli, pname),
            cli::Mode::Test => perform_test(&cli),
        }
    } else {
        Ok(())
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if let Err(e) = do_main(args) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn test_integrity_ok() {
        let file = "testdata/e.txt.bz2";
        assert!(do_main(vec!["bzip2rs", "-t", file]).is_ok());
    }

    #[test]
    fn test_integrity_ng() {
        let file = "testdata/fail-issue5747.bz2.base64";
        assert!(do_main(vec!["bzip2rs", "-t", file]).is_err());
    }

    #[test]
    fn test_decompress() {
        std::fs::copy("testdata/e.txt.bz2", "testdata/e2.txt.bz2")
            .expect("failed to copy test file");
        let r = do_main(vec!["bzip2rs", "testdata/e2.txt.bz2"]);
        assert!(r.is_ok());
        let result = Path::new("testdata/e2.txt");
        assert!(result.exists());
        assert!(result.is_file());
        assert!(! Path::new("testdata/e2.txt.bz2").exists());
        std::fs::remove_file(result)
            .expect("failed to remove test file");
    }

    #[test]
    fn test_compress() {
        std::fs::copy("testdata/alice-in-wonderland.txt", "testdata/alice-in-wonderland-copy.txt")
            .expect("failed to copy test file");
        let r = do_main(vec!["bzip2rs", "testdata/alice-in-wonderland-copy.txt"]);
        assert!(r.is_ok());
        let result = Path::new("testdata/alice-in-wonderland-copy.txt.bz2");
        assert!(result.exists());
        assert!(result.is_file());
        assert!(! Path::new("testdata/alice-in-wonderland-copy.txt").exists());
        std::fs::remove_file(result)
            .expect("failed to remove test file");
    }

    #[test]
    fn test_compress_and_decompress() {
        std::fs::copy("testdata/alice-in-wonderland.txt", "testdata/alice-in-wonderland-copy2.txt")
            .expect("failed to copy test file");
        let r = do_main(vec!["bzip2rs", "testdata/alice-in-wonderland-copy2.txt"]);
        assert!(r.is_ok());
        assert!(Path::new("testdata/alice-in-wonderland-copy2.txt.bz2").exists());
        assert!(! Path::new("testdata/alice-in-wonderland-copy2.txt").exists());
        let r = do_main(vec!["bzip2rs", "testdata/alice-in-wonderland-copy2.txt.bz2"]);
        assert!(r.is_ok());
        assert!(! Path::new("testdata/alice-in-wonderland-copy2.txt.bz2").exists());
        assert!(Path::new("testdata/alice-in-wonderland-copy2.txt").exists());

        let expected = std::fs::read_to_string("testdata/alice-in-wonderland.txt")
            .expect("failed to read test file");
        let actual = std::fs::read_to_string("testdata/alice-in-wonderland-copy2.txt")
            .expect("failed to read test file");
        assert_eq!(expected, actual);

        std::fs::remove_file("testdata/alice-in-wonderland-copy2.txt")
            .expect("failed to remove test file");
    }
}