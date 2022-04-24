fn main() -> Result<(), Box<dyn std::error::Error>> {
    rosetta_build::config()
        .source("fr", "./locales/fr.json")
        .fallback("fr")
        .generate()?;

    Ok(())
}
