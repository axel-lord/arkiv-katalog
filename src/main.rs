#![doc = include_str!("../README.md")]

use ::clap::Parser;
use ::log::LevelFilter;
use ::mimalloc::MiMalloc;

/// Use mimalloc as global allocator
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

fn main() -> ::color_eyre::Result<()> {
    ::env_logger::builder()
        .filter_module("arkiv_katalog", LevelFilter::Info)
        .init();
    ::color_eyre::install()?;
    ::arkiv_katalog::Cli::parse().run()
}
