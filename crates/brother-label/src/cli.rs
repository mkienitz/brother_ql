use std::num::NonZeroU8;
use std::path::PathBuf;

use brother_ql::{media::Media, printer::PrinterModel};
use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Brother QL label printer CLI",
    long_about = "A command-line interface for printing labels and managing Brother QL series label printers.\n\nSupports printing via USB connection, kernel device drivers, and reading printer status."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long, help = "Enable debug logging output")]
    pub debug: bool,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
#[command(next_help_heading = "Printer Connection")]
pub struct PrinterSelection {
    #[arg(
        short,
        long,
        value_name = "MODEL",
        help = "Connect to USB printer with specified model"
    )]
    pub usb: Option<PrinterModel>,

    #[arg(
        long,
        help = "Automatically discover and connect to first available USB printer"
    )]
    pub usb_auto_discover: bool,

    #[arg(
        short,
        long,
        value_name = "DEVICE",
        help = "Connect via kernel device driver (e.g., /dev/usb/lp0)"
    )]
    pub fd: Option<PathBuf>,
}

#[derive(Args, Debug)]
#[command(next_help_heading = "Image Selection")]
#[group(required = true, multiple = false)]
pub struct ImageSelection {
    #[arg(
        short,
        long,
        value_name = "FILE(s)",
        num_args = 1..,
        help = "Path(s) to image file (PNG, JPEG, etc.)"
    )]
    pub images: Option<Vec<PathBuf>>,

    #[arg(long, help = "Use a test label showing media dimensions")]
    pub use_test_image: bool,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
#[command(next_help_heading = "Media Selection")]
pub struct MediaSelection {
    #[arg(short, long, value_enum, help = "Label media type")]
    pub media: Option<Media>,

    #[arg(long, help = "Infer media type from printer status")]
    pub infer_media: bool,
}

#[derive(Args, Debug)]
#[command(next_help_heading = "Print options")]
pub struct PrintOptions {
    #[arg(
        short,
        long,
        value_name = "COUNT",
        help = "Number of copies to print",
        default_value_t = NonZeroU8::MIN
    )]
    pub copies: NonZeroU8,

    #[arg(
            long,
            help = "Prioritize quality over speed",
            default_value = "true",
            default_missing_value = "true",
            num_args = 0..=1,
            require_equals = true
        )]
    pub quality_priority: Option<bool>,

    #[arg(long, value_enum, help = "Cut behavior")]
    pub cut_behavior: Option<CutBehavior>,

    #[arg(
        long,
        value_name = "N",
        help = "Cut every N pages (alternative to --cut-behavior)",
        conflicts_with = "cut_behavior"
    )]
    pub cut_every: Option<std::num::NonZeroU8>,

    #[arg(
            long,
            help = "Use double the resolution along the feeding direction.\nNOTE: this requires supplying an adjusted image.",
            conflicts_with = "use_test_image",
            default_value = "false",
            default_missing_value = "false",
            num_args = 0..=1,
            require_equals = true
        )]
    pub high_dpi: Option<bool>,

    #[arg(
            long,
            help = "Use print data compression (currently has no effect)",
            default_value = "false",
            default_missing_value = "false",
            num_args = 0..=1,
            require_equals = true
        )]
    pub compress: Option<bool>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Print a label to the printer
    Print {
        #[command(flatten)]
        printer: PrinterSelection,

        #[command(flatten)]
        images: ImageSelection,

        #[command(flatten)]
        media_selection: MediaSelection,

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
pub enum CutBehavior {
    NoCut,
    CutEach,
    CutAtEnd,
}

impl CutBehavior {
    pub fn to_unwrapped(self) -> brother_ql::printjob::CutBehavior {
        use brother_ql::printjob::CutBehavior as CB;
        match self {
            CutBehavior::NoCut => CB::NoCut,
            CutBehavior::CutEach => CB::CutEach,
            CutBehavior::CutAtEnd => CB::CutAtEnd,
        }
    }
}
