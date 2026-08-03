use anyhow::{Context, Result, anyhow};
use brother_ql::{
    media::Media,
    printjob::{PrintJob, PrintJobBuilder},
    test_labels::render_test_label,
};

use crate::{
    cli::{Command, ImageSource, MediaSource, PrintArgs, PrintOptions, StatusArgs},
    connection::{BidirectionalConnection, PrintTarget},
};

pub(crate) fn run(command: Command) -> Result<()> {
    match command {
        Command::Print(args) => print(args),
        Command::Status(args) => status(args),
    }
}

fn print(args: PrintArgs) -> Result<()> {
    let PrintArgs {
        target,
        images,
        media,
        options,
    } = args;

    if options.compress {
        eprintln!("Warning: --compress currently has no effect");
    }

    let mut target = PrintTarget::open(target.into_selection())?;
    let media = resolve_media(media.into_source(), &mut target)?;
    let job = build_print_job(media, images.into_source(), &options)?;
    target.print(job)
}

fn status(args: StatusArgs) -> Result<()> {
    let mut connection = BidirectionalConnection::open(args.into_connection())?;
    print!("{}", connection.get_status()?);
    Ok(())
}

fn resolve_media(source: MediaSource, target: &mut PrintTarget) -> Result<Media> {
    match (source, target) {
        (MediaSource::Explicit(media), _) => Ok(media),
        (MediaSource::Infer, PrintTarget::Bidirectional(connection)) => infer_media(connection),
        (MediaSource::Infer, PrintTarget::Network(_)) => {
            unreachable!("--infer-media conflicts with --network in clap")
        }
    }
}

fn infer_media(connection: &mut BidirectionalConnection) -> Result<Media> {
    let status = connection.get_status()?;
    let label_type = status
        .media_type
        .ok_or_else(|| anyhow!("printer did not report a media type"))?;

    Media::from_status_info(label_type, status.media_width, status.media_length).ok_or_else(|| {
        anyhow!(
            "could not identify media from printer status \
             (type={label_type:?}, width={}mm, length={}mm)",
            status.media_width,
            status.media_length,
        )
    })
}

fn build_print_job(media: Media, source: ImageSource, options: &PrintOptions) -> Result<PrintJob> {
    let mut builder = match source {
        ImageSource::Files(paths) => {
            let mut images = paths
                .into_iter()
                .map(|path| {
                    image::open(&path)
                        .with_context(|| format!("could not open image {}", path.display()))
                })
                .collect::<Result<Vec<_>>>()?
                .into_iter();

            PrintJobBuilder::new(media)
                .add_label(
                    images
                        .next()
                        .expect("at least one image path is required by clap"),
                )
                .add_labels(images)
        }
        ImageSource::TestImage => PrintJobBuilder::new(media).add_label(render_test_label(media)?),
    };

    builder = builder
        .copies(options.copies)
        .high_dpi(options.high_dpi)
        .compressed(options.compress)
        .quality_priority(options.quality_priority);

    // Preserve the builder's media-dependent default unless the user selected a cut mode.
    if let Some(cut_behavior) = options.cut_behavior() {
        builder = builder.cut_behavior(cut_behavior);
    }

    Ok(builder.build()?)
}
