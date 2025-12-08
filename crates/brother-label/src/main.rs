use std::path::PathBuf;

use anyhow::{Result, anyhow};
use brother_ql::{
    connection::{KernelConnection, PrinterConnection, UsbConnection, UsbConnectionInfo},
    media::Media,
    printer::PrinterModel,
    printjob::PrintJobBuilder,
    test_labels::render_test_label,
};
use clap::{Args, Parser, Subcommand, ValueEnum};
use tracing_subscriber::{EnvFilter, field::MakeExt};

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Brother QL label printer CLI",
    long_about = "A command-line interface for printing labels and managing Brother QL series label printers.\n\nSupports printing via USB connection, kernel device drivers, and reading printer status."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, help = "Enable debug logging output")]
    debug: bool,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
#[command(next_help_heading = "Printer Connection")]
struct PrinterSelection {
    #[arg(
        short,
        long,
        value_name = "MODEL",
        help = "Connect to USB printer with specified model"
    )]
    usb: Option<PrinterModel>,

    #[arg(
        long,
        help = "Automatically discover and connect to first available USB printer"
    )]
    usb_auto_discover: bool,

    #[arg(
        short,
        long,
        value_name = "DEVICE",
        help = "Connect via kernel device driver (e.g., /dev/usb/lp0)"
    )]
    fd: Option<PathBuf>,
}

#[derive(Args, Debug)]
#[command(next_help_heading = "Image Selection")]
#[group(required = true, multiple = false)]
struct ImageSelection {
    #[arg(
        short,
        long,
        value_name = "FILE(s)",
        num_args = 1..,
        help = "Path(s) to image file (PNG, JPEG, etc.)"
    )]
    images: Option<Vec<PathBuf>>,

    #[arg(long, help = "Use a test label showing media dimensions")]
    use_test_image: bool,
}

#[derive(Args, Debug)]
#[command(next_help_heading = "Print options")]
struct PrintOptions {
    #[arg(short, long, value_enum, help = "Label media type")]
    media: Media,

    #[arg(
        short,
        long,
        group = "options",
        value_name = "COUNT",
        help = "Number of copies to print",
        default_value_t = 1
    )]
    copies: u8,

    #[arg(
            long,
            group = "options",
            help = "Prioritize quality over speed",
            default_value = "true",
            default_missing_value = "true",
            num_args = 0..=1,
            require_equals = true
        )]
    quality_priority: Option<bool>,

    #[arg(long, value_enum, help = "Cut behavior")]
    cut_behavior: Option<CutBehavior>,

    #[arg(
            long,
            group = "options",
            help = "Use double the resolution along the feeding direction.\nNOTE: this requires supplying an adjusted image.",
            conflicts_with = "use_test_image",
            default_value = "false",
            default_missing_value = "false",
            num_args = 0..=1,
            require_equals = true
        )]
    high_dpi: Option<bool>,

    #[arg(
            long,
            group = "options",
            help = "Use print data compression (currently has no effect)",
            default_value = "false",
            default_missing_value = "false",
            num_args = 0..=1,
            require_equals = true
        )]
    compress: Option<bool>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Print a label to the printer
    Print {
        #[command(flatten)]
        printer: PrinterSelection,

        #[command(flatten)]
        images: ImageSelection,

        #[command(flatten)]
        print_options: PrintOptions,
    },
    /// Read and display printer status information
    Status {
        #[command(flatten)]
        printer: PrinterSelection,
    },
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum CutBehavior {
    NoCut,
    CutEach,
    CutAtEnd,
}

// Helper because the library crate type is not primitive
impl CutBehavior {
    fn to_unwrapped(self) -> brother_ql::printjob::CutBehavior {
        use brother_ql::printjob::CutBehavior as CB;
        match self {
            CutBehavior::NoCut => CB::None,
            CutBehavior::CutEach => CB::CutEach,
            CutBehavior::CutAtEnd => CB::CutAtEnd,
        }
    }
}

enum Connection {
    Usb(UsbConnection),
    Kernel(KernelConnection),
}

// Connection helpers
impl Connection {
    fn print(&mut self, job: brother_ql::printjob::PrintJob) -> Result<()> {
        match self {
            Connection::Usb(conn) => conn.print(job)?,
            Connection::Kernel(conn) => conn.print(job)?,
        };
        Ok(())
    }

    fn get_status(&mut self) -> Result<brother_ql::status::StatusInformation> {
        Ok(match self {
            Connection::Usb(conn) => conn.get_status()?,
            Connection::Kernel(conn) => conn.get_status()?,
        })
    }
}

fn create_connection(printer: PrinterSelection) -> Result<Connection> {
    match (printer.usb, printer.fd, printer.usb_auto_discover) {
        (Some(printer_model), _, _) => Ok(Connection::Usb(UsbConnection::open(
            UsbConnectionInfo::from_model(printer_model),
        )?)),
        (_, Some(path), _) => Ok(Connection::Kernel(KernelConnection::open(path)?)),
        (_, _, true) => {
            let conn_info = UsbConnectionInfo::discover()?
                .ok_or_else(|| anyhow!("Couldn't auto-discover any printers!"))?;
            Ok(Connection::Usb(UsbConnection::open(conn_info)?))
        }
        _ => unreachable!(),
    }
}

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
            print_options,
        } => {
            // TODO: remove warning once brother_ql implements compression
            if let Some(true) = print_options.compress {
                println!("Warning: --compress currently has no effect")
            }
            // Get images
            let mut pj_builder = match (images.images, images.use_test_image) {
                (Some(paths), _) => {
                    let imgs = paths
                        .into_iter()
                        .map(|p| image::open(p).map_err(|e| anyhow!("{e}")))
                        .collect::<Result<Vec<_>>>()?;
                    let mut it = imgs.into_iter();
                    PrintJobBuilder::new(print_options.media)
                        .add_label(
                            it.next()
                                .expect("Empty image file list! This should be guarded by clap!"),
                        )
                        .add_labels(it)
                }
                (_, true) => PrintJobBuilder::new(print_options.media)
                    .add_label(render_test_label(print_options.media)?),
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
            }

            let pj = pj_builder.build()?;

            // Get printer connection and print
            let mut conn = create_connection(printer)?;
            conn.print(pj)?;
        }
        Commands::Status { printer } => {
            // Get printer connection and status
            let mut conn = create_connection(printer)?;
            let status = conn.get_status()?;
            println!("{:#?}", status);
        }
    }
    Ok(())
}
