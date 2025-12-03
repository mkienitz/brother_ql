//! Generate test labels for all media types using Typst.
//! Run this code if you want to inspect the current test labels.

use std::error::Error;

use brother_ql::{media::Media, test_labels::render_test_label};
use strum::IntoEnumIterator;
use tracing_subscriber::{field::MakeExt, EnvFilter};

fn main() -> Result<(), Box<dyn Error>> {
    // This example uses pretty logging
    tracing_subscriber::fmt()
        .map_fmt_fields(MakeExt::debug_alt)
        .with_env_filter(EnvFilter::new("debug"))
        .init();

    // Generate test labels for all media types
    for media in Media::iter() {
        let img = render_test_label(media)?;
        img.save(format!("assets/{media}.png"))?;
    }
    Ok(())
}
