use std::{num::NonZeroU8, path::PathBuf};

use brother_ql::{media::Media, printer::PrinterModel};
use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum, ValueHint};

use crate::connection::{BidirectionalConnectionSelection, NetworkEndpoint, PrintTargetSelection};

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Brother QL label printer CLI",
    long_about = "A command-line interface for printing labels and managing Brother QL series label printers.\n\nSupports printing via USB, kernel device drivers, and raw TCP network connections, plus reading status over bidirectional connections."
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,

    #[arg(long, global = true, help = "Enable debug logging output")]
    pub(crate) debug: bool,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Print one or more labels
    Print(PrintArgs),
    /// Read and display printer status information
    Status(StatusArgs),
}

#[derive(Args, Debug)]
pub(crate) struct PrintArgs {
    #[command(flatten)]
    pub(crate) target: PrintTargetArgs,

    #[command(flatten)]
    pub(crate) images: ImageArgs,

    #[command(flatten)]
    pub(crate) media: MediaArgs,

    #[command(flatten)]
    pub(crate) options: PrintOptions,
}

#[derive(Args, Debug)]
pub(crate) struct StatusArgs {
    #[command(flatten)]
    connection: StatusConnectionArgs,
}

impl StatusArgs {
    pub(crate) fn into_connection(self) -> BidirectionalConnectionSelection {
        self.connection.into_selection()
    }
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
#[command(next_help_heading = "Printer Connection")]
pub(crate) struct PrintTargetArgs {
    #[arg(
        short,
        long,
        value_name = "MODEL",
        help = "Connect to a USB printer with the specified model"
    )]
    usb: Option<PrinterModel>,

    #[arg(
        long,
        help = "Automatically discover and connect to the first available USB printer"
    )]
    usb_auto_discover: bool,

    #[arg(
        short = 'f',
        long = "device",
        visible_alias = "fd",
        value_name = "PATH",
        value_hint = ValueHint::FilePath,
        help = "Connect through a kernel printer device (e.g. /dev/usb/lp0)"
    )]
    device: Option<PathBuf>,

    #[arg(
        long,
        value_name = "HOST[:PORT]",
        requires = "media",
        help = "Connect to a network printer (default port: 9100)"
    )]
    network: Option<NetworkEndpoint>,
}

impl PrintTargetArgs {
    pub(crate) fn into_selection(self) -> PrintTargetSelection {
        match (self.usb, self.usb_auto_discover, self.device, self.network) {
            (Some(model), _, _, _) => {
                PrintTargetSelection::Bidirectional(BidirectionalConnectionSelection::Usb(model))
            }
            (_, true, _, _) => PrintTargetSelection::Bidirectional(
                BidirectionalConnectionSelection::UsbAutoDiscover,
            ),
            (_, _, Some(path), _) => {
                PrintTargetSelection::Bidirectional(BidirectionalConnectionSelection::Device(path))
            }
            (_, _, _, Some(endpoint)) => PrintTargetSelection::Network(endpoint),
            _ => unreachable!("print target selection is enforced by clap"),
        }
    }
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
#[command(next_help_heading = "Printer Connection")]
struct StatusConnectionArgs {
    #[arg(
        short,
        long,
        value_name = "MODEL",
        help = "Connect to a USB printer with the specified model"
    )]
    usb: Option<PrinterModel>,

    #[arg(
        long,
        help = "Automatically discover and connect to the first available USB printer"
    )]
    usb_auto_discover: bool,

    #[arg(
        short = 'f',
        long = "device",
        visible_alias = "fd",
        value_name = "PATH",
        value_hint = ValueHint::FilePath,
        help = "Connect through a kernel printer device (e.g. /dev/usb/lp0)"
    )]
    device: Option<PathBuf>,
}

impl StatusConnectionArgs {
    fn into_selection(self) -> BidirectionalConnectionSelection {
        match (self.usb, self.usb_auto_discover, self.device) {
            (Some(model), _, _) => BidirectionalConnectionSelection::Usb(model),
            (_, true, _) => BidirectionalConnectionSelection::UsbAutoDiscover,
            (_, _, Some(path)) => BidirectionalConnectionSelection::Device(path),
            _ => unreachable!("status connection selection is enforced by clap"),
        }
    }
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
#[command(next_help_heading = "Image Selection")]
pub(crate) struct ImageArgs {
    #[arg(
        value_name = "IMAGE",
        num_args = 1..,
        value_hint = ValueHint::FilePath,
        help = "Image file(s) to print (PNG, JPEG, etc.)"
    )]
    images: Option<Vec<PathBuf>>,

    #[arg(long, help = "Generate a test label showing the media dimensions")]
    use_test_image: bool,
}

impl ImageArgs {
    pub(crate) fn into_source(self) -> ImageSource {
        match (self.images, self.use_test_image) {
            (Some(paths), _) => ImageSource::Files(paths),
            (_, true) => ImageSource::TestImage,
            _ => unreachable!("image selection is enforced by clap"),
        }
    }
}

pub(crate) enum ImageSource {
    Files(Vec<PathBuf>),
    TestImage,
}

#[derive(Args, Debug)]
#[group(required = true, multiple = false)]
#[command(next_help_heading = "Media Selection")]
pub(crate) struct MediaArgs {
    #[arg(short, long, value_enum, help = "Label media type")]
    media: Option<Media>,

    #[arg(
        long,
        conflicts_with = "network",
        help = "Infer the media type from printer status"
    )]
    infer_media: bool,
}

impl MediaArgs {
    pub(crate) fn into_source(self) -> MediaSource {
        match (self.media, self.infer_media) {
            (Some(media), _) => MediaSource::Explicit(media),
            (_, true) => MediaSource::Infer,
            _ => unreachable!("media selection is enforced by clap"),
        }
    }
}

pub(crate) enum MediaSource {
    Explicit(Media),
    Infer,
}

#[derive(Args, Debug)]
#[command(next_help_heading = "Print Options")]
pub(crate) struct PrintOptions {
    #[arg(
        short,
        long,
        value_name = "COUNT",
        default_value_t = NonZeroU8::MIN,
        help = "Number of copies to print"
    )]
    pub(crate) copies: NonZeroU8,

    #[arg(
        long,
        action = ArgAction::Set,
        default_value_t = true,
        default_missing_value = "true",
        num_args = 0..=1,
        require_equals = true,
        help = "Prioritize quality over speed"
    )]
    pub(crate) quality_priority: bool,

    #[arg(long, value_enum, help = "Automatic cutter behavior")]
    cut_behavior: Option<CutBehavior>,

    #[arg(
        long,
        value_name = "N",
        conflicts_with = "cut_behavior",
        help = "Cut after every N pages"
    )]
    cut_every: Option<NonZeroU8>,

    #[arg(
        long,
        action = ArgAction::Set,
        default_value_t = false,
        default_missing_value = "true",
        num_args = 0..=1,
        require_equals = true,
        conflicts_with = "use_test_image",
        help = "Use double resolution along the media feed direction"
    )]
    pub(crate) high_dpi: bool,

    #[arg(
        long,
        action = ArgAction::Set,
        default_value_t = false,
        default_missing_value = "true",
        num_args = 0..=1,
        require_equals = true,
        help = "Use print-data compression (currently has no effect)"
    )]
    pub(crate) compress: bool,
}

impl PrintOptions {
    pub(crate) fn cut_behavior(&self) -> Option<brother_ql::printjob::CutBehavior> {
        self.cut_behavior.map(CutBehavior::into_core).or_else(|| {
            self.cut_every
                .map(brother_ql::printjob::CutBehavior::CutEvery)
        })
    }
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum CutBehavior {
    NoCut,
    CutEach,
    CutAtEnd,
}

impl CutBehavior {
    fn into_core(self) -> brother_ql::printjob::CutBehavior {
        match self {
            Self::NoCut => brother_ql::printjob::CutBehavior::NoCut,
            Self::CutEach => brother_ql::printjob::CutBehavior::CutEach,
            Self::CutAtEnd => brother_ql::printjob::CutBehavior::CutAtEnd,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use clap::{CommandFactory, Parser, error::ErrorKind};

    use super::{Cli, Command, ImageSource};
    use crate::connection::{BidirectionalConnectionSelection, PrintTargetSelection};

    const BASE_PRINT: [&str; 8] = [
        "brother-label",
        "print",
        "label.png",
        "--media",
        "c62",
        "--network",
        "printer.local",
        "--quality-priority=false",
    ];

    #[test]
    fn clap_configuration_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn positional_images_and_boolean_values_parse() {
        let cli = Cli::try_parse_from(BASE_PRINT).unwrap();
        let Command::Print(args) = cli.command else {
            panic!("expected print command");
        };

        assert!(!args.options.quality_priority);
        assert!(!args.options.high_dpi);
        assert!(!args.options.compress);
        assert!(matches!(
            args.images.into_source(),
            ImageSource::Files(paths) if paths == [PathBuf::from("label.png")]
        ));
        assert!(matches!(
            args.target.into_selection(),
            PrintTargetSelection::Network(endpoint) if endpoint.to_string() == "printer.local:9100"
        ));
    }

    #[test]
    fn multiple_positional_images_parse() {
        let cli = Cli::try_parse_from([
            "brother-label",
            "print",
            "first.png",
            "second.png",
            "--media",
            "c62",
            "--usb-auto-discover",
        ])
        .unwrap();
        let Command::Print(args) = cli.command else {
            panic!("expected print command");
        };

        assert!(matches!(
            args.images.into_source(),
            ImageSource::Files(paths)
                if paths == [PathBuf::from("first.png"), PathBuf::from("second.png")]
        ));
    }

    #[test]
    fn bare_boolean_flags_enable_features() {
        let cli = Cli::try_parse_from([
            "brother-label",
            "print",
            "label.png",
            "--media",
            "c62",
            "--network",
            "printer.local",
            "--high-dpi",
            "--compress",
        ])
        .unwrap();
        let Command::Print(args) = cli.command else {
            panic!("expected print command");
        };

        assert!(args.options.high_dpi);
        assert!(args.options.compress);
        assert!(args.options.quality_priority);
    }

    #[test]
    fn explicit_false_boolean_values_are_supported() {
        let cli = Cli::try_parse_from([
            "brother-label",
            "print",
            "label.png",
            "--media",
            "c62",
            "--network",
            "printer.local",
            "--high-dpi=false",
            "--compress=false",
        ])
        .unwrap();
        let Command::Print(args) = cli.command else {
            panic!("expected print command");
        };

        assert!(!args.options.high_dpi);
        assert!(!args.options.compress);
    }

    #[test]
    fn network_conflicts_with_media_inference() {
        let error = Cli::try_parse_from([
            "brother-label",
            "print",
            "label.png",
            "--network",
            "printer.local",
            "--infer-media",
        ])
        .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::ArgumentConflict);
    }

    #[test]
    fn printer_image_media_and_cut_selections_are_exclusive() {
        for arguments in [
            vec![
                "brother-label",
                "print",
                "label.png",
                "--media",
                "c62",
                "--network",
                "printer.local",
                "--usb-auto-discover",
            ],
            vec![
                "brother-label",
                "print",
                "label.png",
                "--use-test-image",
                "--media",
                "c62",
                "--usb-auto-discover",
            ],
            vec![
                "brother-label",
                "print",
                "label.png",
                "--media",
                "c62",
                "--infer-media",
                "--usb-auto-discover",
            ],
            vec![
                "brother-label",
                "print",
                "label.png",
                "--media",
                "c62",
                "--usb-auto-discover",
                "--cut-behavior",
                "cut-each",
                "--cut-every",
                "2",
            ],
        ] {
            assert_eq!(
                Cli::try_parse_from(arguments).unwrap_err().kind(),
                ErrorKind::ArgumentConflict
            );
        }
    }

    #[test]
    fn generated_test_image_conflicts_with_high_dpi() {
        let error = Cli::try_parse_from([
            "brother-label",
            "print",
            "--use-test-image",
            "--media",
            "c62",
            "--usb-auto-discover",
            "--high-dpi",
        ])
        .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::ArgumentConflict);
    }

    #[test]
    fn copies_cut_interval_and_network_port_must_be_nonzero() {
        for arguments in [
            vec![
                "brother-label",
                "print",
                "label.png",
                "--media",
                "c62",
                "--usb-auto-discover",
                "--copies",
                "0",
            ],
            vec![
                "brother-label",
                "print",
                "label.png",
                "--media",
                "c62",
                "--usb-auto-discover",
                "--cut-every",
                "0",
            ],
            vec![
                "brother-label",
                "print",
                "label.png",
                "--media",
                "c62",
                "--network",
                "printer.local:0",
            ],
        ] {
            assert_eq!(
                Cli::try_parse_from(arguments).unwrap_err().kind(),
                ErrorKind::ValueValidation
            );
        }
    }

    #[test]
    fn device_name_alias_and_short_option_parse() {
        for option in ["--device", "--fd", "-f"] {
            let cli =
                Cli::try_parse_from(["brother-label", "status", option, "/dev/usb/lp0"]).unwrap();
            let Command::Status(args) = cli.command else {
                panic!("expected status command");
            };
            assert!(matches!(
                args.into_connection(),
                BidirectionalConnectionSelection::Device(path)
                    if path.as_path() == Path::new("/dev/usb/lp0")
            ));
        }
    }

    #[test]
    fn status_rejects_network_selection() {
        let error = Cli::try_parse_from(["brother-label", "status", "--network", "printer.local"])
            .unwrap_err();
        assert_eq!(error.kind(), ErrorKind::UnknownArgument);
    }

    #[test]
    fn debug_is_global() {
        for arguments in [
            ["brother-label", "--debug", "status", "--usb-auto-discover"],
            ["brother-label", "status", "--usb-auto-discover", "--debug"],
        ] {
            assert!(Cli::try_parse_from(arguments).unwrap().debug);
        }
    }
}
