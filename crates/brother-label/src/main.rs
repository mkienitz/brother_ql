mod cli;
mod connection;

use anyhow::{Result, anyhow};
use brother_ql::{media::Media, printjob::PrintJobBuilder, test_labels::render_test_label};
use clap::Parser;
use tracing_subscriber::{EnvFilter, field::MakeExt};

use cli::{Cli, Commands};
use connection::create_connection;

fn main() -> Result<()> {
    let cli = Cli::parse();
    tracing_subscriber::fmt()
        .map_fmt_fields(MakeExt::debug_alt)
        .with_env_filter(EnvFilter::new(if cli.debug { "debug" } else { "info" }))
        .init();
    match cli.command {
        Commands::Print {
            printer,
            images,
            media_selection,
            print_options,
        } => {
            // TODO: remove warning once brother_ql implements compression
            if let Some(true) = print_options.compress {
                println!("Warning: --compress currently has no effect")
            }

            // Resolve media: either from --media or by inferring from printer status
            let mut conn = create_connection(printer)?;
            let media = if let Some(media) = media_selection.media {
                media
            } else {
                let status = conn.get_status()?;
                let label_type = status
                    .media_type
                    .ok_or_else(|| anyhow!("Printer did not report a media type"))?;
                Media::from_status_info(label_type, status.media_width, status.media_length)
                    .ok_or_else(|| {
                        anyhow!(
                            "Could not identify media from printer status \
                             (type={label_type:?}, width={}mm, length={}mm)",
                            status.media_width,
                            status.media_length,
                        )
                    })?
            };

            // Get images
            let mut pj_builder = match (images.images, images.use_test_image) {
                (Some(paths), _) => {
                    let imgs = paths
                        .into_iter()
                        .map(|p| image::open(p).map_err(|e| anyhow!("{e}")))
                        .collect::<Result<Vec<_>>>()?;
                    let mut it = imgs.into_iter();
                    PrintJobBuilder::new(media)
                        .add_label(
                            it.next()
                                .expect("Empty image file list! This should be guarded by clap!"),
                        )
                        .add_labels(it)
                }
                (_, true) => PrintJobBuilder::new(media).add_label(render_test_label(media)?),
                _ => unreachable!(),
            };

            // Create print job
            pj_builder = pj_builder
                .copies(print_options.copies)
                .high_dpi(
                    print_options
                        .high_dpi
                        .expect("No high-dpi option set! This should be guarded by clap!"),
                )
                .compressed(
                    print_options
                        .compress
                        .expect("No compression option set! This should be guarded by clap!"),
                )
                .quality_priority(
                    print_options
                        .quality_priority
                        .expect("No quality priority set! This should be guarded by clap!"),
                );

            // For cutting behavior, let the builder pick the media-type dependent defaults.
            // Therefore, don't set defaults using unwrap_or at this level.
            if let Some(cb) = print_options.cut_behavior {
                pj_builder = pj_builder.cut_behavior(cb.to_unwrapped());
            } else if let Some(n) = print_options.cut_every {
                pj_builder =
                    pj_builder.cut_behavior(brother_ql::printjob::CutBehavior::CutEvery(n));
            }

            let pj = pj_builder.build()?;
            conn.print(pj)?;
        }
        Commands::Status { printer } => {
            // Get printer connection and status
            let mut conn = create_connection(printer)?;
            let status = conn.get_status()?;
            print!("{status}");
        }
    }
    Ok(())
}
