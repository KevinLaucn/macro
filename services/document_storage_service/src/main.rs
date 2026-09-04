#![recursion_limit = "256"]

#[cfg(feature = "full-product")]
mod full_main;
#[cfg(not(feature = "full-product"))]
mod lean_main;

#[cfg(feature = "full-product")]
fn main() -> anyhow::Result<()> {
    full_main::main()
}

#[cfg(not(feature = "full-product"))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    lean_main::run().await
}
