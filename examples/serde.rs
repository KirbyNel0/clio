use clio::Clio;

fn main() -> serde_json::Result<()> {
    // read the first argument
    let args = std::env::args();
    let Some(first) = args.into_iter().skip(1).next() else {
        return Ok(());
    };

    // parse argument as `IORedirect`
    let target = Clio::parse(&first);
    println!("Target: {:?}", target);

    // convert the input to JSON
    let ser = serde_json::to_string(&target)?;
    println!("Serialized: {}", ser);

    // convert the input back from JSON
    let de = serde_json::from_str::<Clio>(&ser)?;
    println!("Deserialized: {}", de);

    Ok(())
}
