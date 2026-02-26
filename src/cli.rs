#[derive(clap::Parser, Debug)]
#[command(
    version, propagate_version = true,
    about = "A bzip2-compatible command line interface",
    disable_version_flag = true
)]
pub(crate) struct Bzip2Cli {
    #[clap(short, long, help = "force decompression")]
    pub decompress: bool,
    #[clap(short ='z', long, help = "force compression")]
    pub compress: bool,
    #[clap(short, long, help = "keep (don't delete) input files")]
    pub keep: bool,
    #[clap(short, long, help = "overwrite existing output files")]
    pub force: bool,
    #[clap(short, long, help = "test compressed file integrity")]
    pub test: bool,
    #[clap(short = 'c', long, help = "output to standard out")]
    pub stdout: bool,
    #[clap(short, long, help = "suppress noncritical error messages")]
    pub quiet: bool,
    #[clap(short, long, default_value = "0", help = "be verbose (a 2nd -v gives more)")]
    pub verbose: u8,
    #[clap(short = 'L', long, help = "display software version & license")]
    pub license: bool,
    #[clap(short = 'V', long, help = "display software version & license")]
    pub version: bool,
    #[clap(short, long, help = "use less memory (at most 2500k)")]
    pub small: bool,
    #[clap(short, long, help = "set block size to 100k .. 900k")]
    pub block_size: Option<u8>,
    #[clap(long, help = "alias for -1")]
    pub fast: bool,
    #[clap(long, help = "alias for -9")]
    pub best: bool,
    #[arg(index = 1, help = "input file(s)")]
    pub input_files: Vec<String>,
}

pub(crate) enum Mode {
    Compress,
    Decompress,
    Test,
}

fn init_logger(cli: &Bzip2Cli) {
    unsafe {
        if cli.quiet {
            std::env::set_var("RUST_LOG", "error");
        } else if cli.verbose >= 2 {
            std::env::set_var("RUST_LOG", "trace");
        } else if cli.verbose == 1 {
            std::env::set_var("RUST_LOG", "info");
        } else {
            std::env::set_var("RUST_LOG", "warn");
        }
    }
    env_logger::try_init().unwrap_or_else(|_| {
        eprintln!("failed to initialize logger. set RUST_LOG to see logs.");
    });
    log::info!("set log level to {}", std::env::var("RUST_LOG").unwrap_or_else(|_| "unknown".to_string()));
}

impl Bzip2Cli {
    pub fn init(&self) -> bool {
        if self.license || self.version {
            println!("bzip2rs {}", env!("CARGO_PKG_VERSION"));
            println!("Copyright (C) 2026 by Haruaki Tamada");
            println!("License: MIT");
            return true;
        }
        init_logger(self);
        false
    }

    pub fn is_empty(&self) -> bool {
        self.input_files.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.input_files.iter()
    }

    pub fn is_stdout(&self, program_name: &str) -> bool {
        self.stdout || program_name == "bzcat"
    }

    pub fn compress_level(&self) -> usize {
        if self.fast {
            1
        } else if self.best {
            9
        } else if let Some(level) = self.block_size {
            level as usize
        } else {
            6
        }
    }

    pub fn mode(&self, program_name: &str) -> Mode {
        if self.decompress || program_name == "bunzip2" || program_name == "bzcat" {
            Mode::Decompress
        } else if self.test {
            Mode::Test
        } else if self.input_files.iter().all(|f| f.ends_with(".bz2") || f.ends_with(".tbz2") || f.ends_with(".tbz") || f.ends_with(".tz2")) {
            Mode::Decompress
        } else {
            Mode::Compress
        }
    }
}