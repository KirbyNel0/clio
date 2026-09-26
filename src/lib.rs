//! CLI/O: Read and write standard I/O streams or files dynamically.
//!
//! This crate provides [Clio] to easily handle command line arguments which specify where
//! input comes from and where output should go to.
//!
//! As used by many applications, the convention applies that `'-'` as a file name is treated as
//! reading from standard input/writing to standard output. This can lead to much boilerplate
//! code, which this crate tries to simplify:
//!
//! ```
//! use clio::Clio;
//!
//! let stdin = Clio::parse("-");
//! assert_eq!(stdin, Clio::StdStream);
//!
//! let file = Clio::parse("file.txt");
//! assert_eq!(file, Clio::File("file.txt".into()));
//! ```
//!
//! Reading and writing can be achieved by calling the `reader`/`writer` method:
//!
//! ```
//! use clio::Clio;
//! use std::io::Write;
//!
//! let input = Clio::StdStream;
//! let name: String = input.reader().and_then(std::io::read_to_string)?;
//!
//! let output = Clio::StdStream;
//! output.writer().and_then(|mut writer| write!(writer, "Hello, {}!", name))?;
//! # std::io::Result::Ok(())
//! ```

#[cfg(feature = "serde")]
extern crate serde;

use std::fs::File;
use std::io::{Read, Result as IoResult, Stderr, Stdin, Stdout, Write};
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub enum Clio {
    /// Read from standard input or write to standard output.
    #[default]
    StdStream,
    /// Read from or write to a file.
    File(PathBuf),
}

impl Clio {
    /// Create a reader corresponding to this instance. Files are opened in read-only mode.
    /// May fail if opening the file fails.
    pub fn reader(&self) -> IoResult<InputRedirect> {
        match self {
            Self::StdStream => Ok(InputRedirect::Stdin(std::io::stdin())),
            Self::File(filename) => {
                let file = File::open(filename)?;
                Ok(InputRedirect::File(file))
            }
        }
    }

    /// Create a writer corresponding to this instance. Files are opened in write-only mode and
    /// truncated. If the file does not exist, it will be created. May fail if opening the file
    /// fails.
    ///
    /// To append to files instead of truncating them, use [Self::writer_append].
    pub fn writer(&self) -> IoResult<OutputRedirect> {
        match self {
            Self::StdStream => Ok(OutputRedirect::Stdout(std::io::stdout())),
            Self::File(filename) => {
                let file = File::options()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(filename)?;
                Ok(OutputRedirect::File(file))
            }
        }
    }

    /// Create a writer corresponding to this instance. Files are opened in write-only mode. New
    /// output is appended to a file. If the file does not exist, it will be created. May fail if
    /// opening the file fails.
    ///
    /// Same as [Self::writer] if writing to standard output.
    pub fn writer_append(&self) -> IoResult<OutputRedirect> {
        match self {
            Self::StdStream => Ok(OutputRedirect::Stdout(std::io::stdout())),
            Self::File(filename) => {
                let file = File::options().create(true).append(true).open(filename)?;
                Ok(OutputRedirect::File(file))
            }
        }
    }

    /// Check whether this instance would read from standard input.
    pub const fn is_stdin(&self) -> bool {
        matches!(self, Self::StdStream)
    }

    /// Check whether this instance would read from a file. Use [Self::filename] to retrieve the
    /// file it would read from.
    pub const fn is_file(&self) -> bool {
        matches!(self, Self::File(_))
    }

    /// Parse a string and create a new instance. A `'-'` is interpreted as standard stream while
    /// everything else is parsed as filename.
    pub fn parse(s: &str) -> Self {
        if s == "-" {
            Self::StdStream
        } else {
            Self::File(PathBuf::from(s))
        }
    }

    /// Get the filename of this instance, if it points to a file.
    pub fn filename(&self) -> Option<&std::path::Path> {
        match self {
            Self::StdStream => None,
            Self::File(filename) => Some(filename.as_ref()),
        }
    }

    /// Get the filename of this instance as a string, if it points to a file. May fail
    /// if the filename does not contain valid Unicode.
    #[allow(clippy::result_unit_err)]
    pub fn filename_str(&self) -> Result<Option<&str>, ()> {
        match self {
            Self::StdStream => Ok(None),
            Self::File(filename) => match filename.to_str() {
                None => Err(()),
                Some(s) => Ok(Some(s)),
            },
        }
    }

    /// Modify the filename of this instance, if any.
    pub fn map(self, op: impl FnOnce(PathBuf) -> PathBuf) -> Self {
        match self {
            Self::StdStream => Self::StdStream,
            Self::File(filename) => Self::File(op(filename)),
        }
    }

    /// Modify the filename of this instance, if any.
    pub fn map_ref<P>(&self, op: impl FnOnce(&std::path::Path) -> P) -> Self
    where
        P: Into<PathBuf>,
    {
        match self {
            Self::StdStream => Self::StdStream,
            Self::File(filename) => Self::File(op(filename.as_ref()).into()),
        }
    }
}

impl From<Clio> for Option<PathBuf> {
    fn from(value: Clio) -> Self {
        match value {
            Clio::StdStream => None,
            Clio::File(filename) => Some(filename),
        }
    }
}

impl<T> From<Option<T>> for Clio
where
    T: Into<PathBuf>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            None => Self::StdStream,
            Some(filename) => Self::File(filename.into()),
        }
    }
}

impl std::str::FromStr for Clio {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::parse(s))
    }
}

impl std::fmt::Display for Clio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::StdStream => std::fmt::Write::write_char(f, '-'),
            Self::File(filename) => std::fmt::Display::fmt(&filename.display(), f),
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Clio {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.filename().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Clio {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let option = <Option<PathBuf>>::deserialize(deserializer)?;
        Ok(option.into())
    }
}

/// A reader created by [Clio::reader]. Contains a reader which handles access to either
/// standard input or a file.
pub enum InputRedirect {
    /// A handle to read from standard input.
    Stdin(Stdin),
    /// A handle to read from a file.
    File(File),
}

impl Read for InputRedirect {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Stdin(reader) => reader.read(buf),
            Self::File(reader) => reader.read(buf),
        }
    }
}

impl InputRedirect {
    /// Check whether this reader holds a handle to standard input.
    pub const fn is_stdin(&self) -> bool {
        matches!(self, Self::Stdin(_))
    }

    /// Check whether this reader holds a handle to a file.
    pub const fn is_file(&self) -> bool {
        matches!(self, Self::File(_))
    }
}

/// A writer created by [Clio::writer]. Contains a writer which handles access to either
/// standard (error) output or the file.
pub enum OutputRedirect {
    /// A handle to write to standard output.
    Stdout(Stdout),
    /// A handle to write to standard error output.
    Stderr(Stderr),
    /// A handle to write to a file.
    File(File),
}

impl Write for OutputRedirect {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        match self {
            Self::Stdout(writer) => writer.write(buf),
            Self::Stderr(writer) => writer.write(buf),
            Self::File(writer) => writer.write(buf),
        }
    }

    fn flush(&mut self) -> IoResult<()> {
        match self {
            Self::Stdout(writer) => writer.flush(),
            Self::Stderr(writer) => writer.flush(),
            Self::File(writer) => writer.flush(),
        }
    }
}

impl OutputRedirect {
    /// Make this writer point to standard output if it was pointing to standard error output
    /// previously.
    #[must_use]
    pub fn to_stdout(self) -> Self {
        match self {
            Self::Stdout(stdout) => Self::Stdout(stdout),
            Self::Stderr(_stderr) => Self::Stdout(std::io::stdout()),
            Self::File(filename) => Self::File(filename),
        }
    }

    /// Make this writer point to standard error output if it was pointing to standard output
    /// previously.
    #[must_use]
    pub fn to_stderr(self) -> Self {
        match self {
            Self::Stdout(_stdout) => Self::Stderr(std::io::stderr()),
            Self::Stderr(stderr) => Self::Stderr(stderr),
            Self::File(filename) => Self::File(filename),
        }
    }

    /// Check whether this writer holds a handle to standard output.
    pub const fn is_stdout(&self) -> bool {
        matches!(self, Self::Stdout(_))
    }

    /// Check whether this writer holds a handle to standard error output.
    pub const fn is_stderr(&self) -> bool {
        matches!(self, Self::Stderr(_))
    }

    /// Check whether this writer holds a handle to a file.
    pub const fn is_file(&self) -> bool {
        matches!(self, Self::File(_))
    }
}
