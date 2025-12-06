use std::path::PathBuf;

use anyhow::{Result, anyhow};
use brother_ql::{
    connection::{KernelConnection, PrinterConnection, UsbConnection, UsbConnectionInfo},
    media::Media,
    printer::PrinterModel,
    printjob::PrintJobBuilder,
    test_labels::render_test_label,
};
use clap::{Args, Parser, Subcommand};
use image::DynamicImage;
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

#[derive(Debug, Subcommand)]
enum Commands {
    /// Print a label to the printer
    Print {
        #[command(flatten)]
        printer: PrinterSelection,

        #[command(flatten)]
        images: ImageSelection,

        #[arg(
            short,
            long,
            value_enum,
            help_heading = "Print Options",
            help = "Label media type"
        )]
        media: Media,

        #[arg(
            short,
            long,
            group = "options",
            help_heading = "Print Options",
            value_name = "COUNT",
            help = "Number of copies to print",
            default_value_t = 1
        )]
        copies: u8,

        #[arg(
            long,
            group = "options",
            help_heading = "Print Options",
            help = "Prioritize speed over quality"
        )]
        speed_priority: bool,
    },
    /// Read and display printer status information
    Status {
        #[command(flatten)]
        printer: PrinterSelection,
    },
}

// enum CutBehavior {
//     None,
//     CutEach,
//     CutAtEnd,
// }

enum Connection {
    Usb(UsbConnection),
    Kernel(KernelConnection),
}

impl Connection {
    fn print(&mut self, job: brother_ql::printjob::PrintJob) -> Result<()> {
        match self {
            Connection::Usb(conn) => conn.print(job).map_err(Into::into),
            Connection::Kernel(conn) => conn.print(job).map_err(Into::into),
        }
    }

    fn get_status(&mut self) -> Result<brother_ql::status::StatusInformation> {
        match self {
            Connection::Usb(conn) => conn.get_status().map_err(Into::into),
            Connection::Kernel(conn) => conn.get_status().map_err(Into::into),
        }
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
        .with_env_filter(EnvFilter::new(
            cli.debug.then_some("debug").unwrap_or("info"),
        ))
        .init();
    match cli.command {
        Commands::Print {
            printer,
            media,
            images,
            copies,
            speed_priority,
        } => {
            // Get images
            let pj = match (images.images, images.use_test_image) {
                (Some(paths), _) => {
                    let imgs: Result<Vec<DynamicImage>> = paths
                        .into_iter()
                        .map(|p| image::open(p).map_err(|e| anyhow!("{e}")))
                        .collect();
                    let mut it = imgs?.into_iter();
                    PrintJobBuilder::new(media)
                        .add_label(it.next().ok_or(anyhow!("Empty image file list!"))?)
                        .add_labels(it)
                }
                (_, true) => PrintJobBuilder::new(media).add_label(render_test_label(media)?),
                _ => unreachable!(),
            };

            // Create print job
            let pj = pj
                .copies(copies)
                .quality_priority(!speed_priority)
                .build()?;

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
