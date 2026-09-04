#![recursion_limit = "256"]

#[cfg(feature = "full-product")]
mod api;
mod config;
#[cfg(feature = "full-product")]
mod full_main;
#[cfg(not(feature = "full-product"))]
mod lean_main;
#[cfg(feature = "full-product")]
mod model;
#[cfg(feature = "full-product")]
mod outbound;
#[cfg(feature = "full-product")]
mod service;

#[cfg(feature = "full-product")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    full_main::run().await
}

#[cfg(not(feature = "full-product"))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    lean_main::run().await
}
