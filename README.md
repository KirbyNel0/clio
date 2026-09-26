# CLI/O

This small crate provides utility to read from or write to standard output or files dynamically.
It is meant to be used by command line interfaces, so the developer does not have to bother where the user wants to interact with.

By convention, specifying `-` as file name on command line should interact with standard I/O streams.
With this crate, you do not have to bother about your user's input and can just read/write your data.
This is made possible by the `Clio` enum of this crate:

```rust
use clio::Clio;

fn main() -> std::io::Result<()> {
    let from = Clio::parse("-");
    let input = from.reader().and_then(std::io::read_to_string)?;
    
    let to = Clio::parse("file.txt");
    to.writer().and_then(|mut writer| write!(writer, "Hello, {}", input))?;
    
    Ok(())
}
```

Both integrate well with [clap](https://docs.rs/clap/latest/clap/).
They support the [serde](https://docs.rs/serde/latest/serde) interface if the `serde` feature is enabled.
See the [examples](./examples) for further information.
