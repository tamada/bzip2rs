use std::io::{Read, Write};
use crate::cli::Bzip2Cli;
use crate::{Result};

pub(super) fn test_integrity(reader: impl Read) -> Result<u64> {
    #[cfg(feature = "sys")]
    {
        libbzip2::test_integrity(reader)
    }
#[cfg(not(feature = "sys"))]
    {
        pure_rust::test_integrity(reader)
    }
}

pub(super) fn compress(reader: impl Read, writer: impl Write, cli: &Bzip2Cli) -> Result<u64> {
    #[cfg(feature = "sys")]
    {
        libbzip2::compress(reader, writer, cli)
    }
#[cfg(not(feature = "sys"))]
    {
        pure_rust::compress(reader, writer, cli)
    }
}

pub(super) fn decompress(reader: impl Read, writer: impl Write, cli: &Bzip2Cli) -> Result<u64> {
    #[cfg(feature = "sys")]
    {
        libbzip2::decompress(reader, writer, cli)
    }
#[cfg(not(feature = "sys"))]
    {
        pure_rust::decompress(reader, writer, cli)
    }
}

#[cfg(feature = "sys")]
mod libbzip2 {
    use bzip2::{read::MultiBzDecoder, write::BzEncoder, Compression};
    use std::io::{sink, Read, Write};

    use crate::{Error, Result};
    use crate::cli::Bzip2Cli;

    pub(super) fn test_integrity(reader: impl Read) -> Result<u64> {
        let mut decoder = MultiBzDecoder::new(reader);
        match std::io::copy(&mut decoder, &mut sink()) {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(Error::Io(e)),
        }
    }

    pub(super) fn decompress(reader: impl Read, writer: impl Write, _cli: &Bzip2Cli) -> Result<u64> {
        let mut decoder = MultiBzDecoder::new(reader);
        match std::io::copy(&mut decoder, &mut std::io::BufWriter::new(writer)) {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(Error::Io(e)),
        }
    }

    pub(super) fn compress(reader: impl Read, writer: impl Write, cli: &Bzip2Cli) -> Result<u64>{
        let level = cli.compress_level() as u32;
        let compression = Compression::new(level);
        let mut encoder = BzEncoder::new(writer, compression);
        match std::io::copy(&mut std::io::BufReader::new(reader), &mut encoder) {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(Error::Io(e)),
        }
    }
}


#[cfg(not(feature = "sys"))]
mod pure_rust {
    use bzip2_rs::DecoderReader;
    use std::io::{sink, Read, Write};

    use crate::{Error, Result};
    use crate::cli::Bzip2Cli;

    pub(super) fn test_integrity(reader: impl Read) -> Result<u64> {
        let mut decoder = DecoderReader::new(reader);
        match std::io::copy(&mut decoder, &mut sink()) {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(Error::Io(e)),
        }
    }

    pub(super) fn decompress(reader: impl Read, writer: impl Write, _cli: &Bzip2Cli) -> Result<u64> {
        let mut decoder = DecoderReader::new(reader);
        match std::io::copy(&mut decoder, &mut std::io::BufWriter::new(writer)) {
            Ok(bytes) => Ok(bytes),
            Err(e) => Err(Error::Io(e)),
        }
    }

    pub(super) fn compress(reader: impl Read, writer: impl Write, cli: &Bzip2Cli) -> Result<u64>{
        let level = cli.compress_level();
        let mut reader = std::io::BufReader::new(reader);
        let writer = std::io::BufWriter::new(writer);
        match banzai::encode(&mut reader, writer, level) {
            Ok(bytes) => Ok(bytes as u64),
            Err(e) => Err(Error::Io(e)),
        }
    }

}