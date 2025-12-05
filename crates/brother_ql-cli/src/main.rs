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
use tracing_subscriber::{EnvFilter, field::MakeExt};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(long)]
    debug: bool,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
struct PrinterSelection {
    #[arg(short, long, value_name = "PRINTER_MODEL")]
    usb: Option<PrinterModel>,

    #[arg(long)]
    usb_auto_discover: bool,

    #[arg(short, long, value_name = "FILE")]
    fd: Option<PathBuf>,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
struct ImageSelection {
    #[arg(short, long, value_name = "FILE")]
    image: Option<PathBuf>,

    #[arg(long)]
    use_test_image: bool,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Print {
        #[command(flatten)]
        printer: PrinterSelection,

        #[arg(short, long, value_enum)]
        media: Media,

        #[command(flatten)]
        image: ImageSelection,

        #[arg(short, long, group = "options", value_name = "NUMBER")]
        copies: Option<u8>,

        #[arg(short, long, group = "options")]
        quality_priority: Option<bool>,
    },
    Status {
        #[command(flatten)]
        printer: PrinterSelection,
    },
}

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
    if cli.debug {
        tracing_subscriber::fmt()
            .map_fmt_fields(MakeExt::debug_alt)
            .with_env_filter(EnvFilter::new("debug"))
            .init();
    }
    match cli.command {
        Commands::Print {
            printer,
            media,
            image,
            copies,
            quality_priority,
        } => {
            // Get image
            let (img, use_test_label) = (image.image, image.use_test_image);
            let img = match (img, use_test_label) {
                (Some(path), _) => image::open(path)?,
                (_, true) => render_test_label(media)?,
                _ => unreachable!(),
            };

            // Create print job
            let print_job = PrintJobBuilder::new(media)
                .add_label(img)
                .copies(copies.unwrap_or(1))
                .quality_priority(quality_priority.unwrap_or(true))
                .build()?;

            // Get printer connection and print
            let mut conn = create_connection(printer)?;
            conn.print(print_job)?;
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
