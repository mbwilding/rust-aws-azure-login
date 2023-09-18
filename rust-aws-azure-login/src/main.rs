use anyhow::Result;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        //.json()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(true)
        .with_line_number(true)
        .init();

    web::login::login("default", false, false)?;

    Ok(())
}
