//! Raster command generation for Brother QL printers

use crate::media::Media;

pub(crate) enum DynamicCommandMode {
    Raster,
}

pub(crate) enum ColorPower {
    LowEnergy,
    HighEnergy,
}

/// Various mode settings
#[derive(Debug, Clone, Copy)]
pub(crate) struct VariousModeSettings {
    pub auto_cut: bool,
}

pub(crate) enum RasterCommand {
    Invalidate,
    Initialize,
    SpecifyMarginAmount { margin_size: u16 },
    SwitchDynamicCommandMode { command_mode: DynamicCommandMode },
    SwitchAutomaticStatusNotificationMode { notify: bool },
    RasterGraphicsTransfer { data: Vec<u8> },
    TwoColorRasterGraphicsTransfer { data: Vec<u8>, color_power: ColorPower },
    SelectCompressionMode { tiff_compression: bool },
    SpecifyPageNumber { cut_every: u8 },
    VariousMode(VariousModeSettings),
    ExpandedMode { two_color: bool, cut_at_end: bool, high_dpi: bool },
    PrintInformation {
        media: Media,
        quality_priority: bool,
        recovery_on: bool,
        no_lines: u32,
        first_page: bool,
    },
    Print,
    PrintWithFeed,
}

impl From<RasterCommand> for Vec<u8> {
    fn from(value: RasterCommand) -> Self {
        use RasterCommand as RC;
        match value {
            RC::Invalidate => {
                vec![0u8; 400]
            }
            RC::Initialize => {
                vec![0x1b, 0x40]
            }
            RC::SpecifyMarginAmount { margin_size } => {
                let [n2, n1] = margin_size.to_be_bytes();
                vec![0x1b, 0x69, 0x64, n1, n2]
            }
            RC::SwitchDynamicCommandMode { command_mode } => {
                let m = match command_mode {
                    DynamicCommandMode::Raster => 0x01,
                };
                vec![0x1b, 0x69, 0x61, m]
            }
            RC::SwitchAutomaticStatusNotificationMode { notify } => {
                let n = u8::from(!notify);
                vec![0x1b, 0x69, 0x21, n]
            }
            RC::RasterGraphicsTransfer { mut data } => {
                #[allow(clippy::cast_possible_truncation)]
                let mut res = vec![0x67, 0x00, data.len() as u8];
                res.append(&mut data);
                res
            }
            RC::TwoColorRasterGraphicsTransfer { mut data, color_power } => {
                let cp = match color_power {
                    ColorPower::HighEnergy => 0x01,
                    ColorPower::LowEnergy => 0x02,
                };
                #[allow(clippy::cast_possible_truncation)]
                let mut res = vec![0x77, cp, data.len() as u8];
                res.append(&mut data);
                res
            }
            RC::Print => {
                vec![0x0c]
            }
            RC::PrintWithFeed => {
                vec![0x1a]
            }
            RC::SelectCompressionMode { tiff_compression } => {
                let cm = if tiff_compression { 0x02 } else { 0x00 };
                vec![0x4d, cm]
            }
            RC::SpecifyPageNumber { cut_every } => {
                vec![0x1b, 0x69, 0x41, cut_every]
            }
            RC::VariousMode(various_mode) => {
                let vm = if various_mode.auto_cut {
                    0b1 << (7 - 1)
                } else {
                    0x00
                };
                vec![0x1b, 0x69, 0x4d, vm]
            }
            RC::ExpandedMode { two_color, cut_at_end, high_dpi } => {
                let mut flags = 0x00;
                if two_color {
                    flags |= 0b1;
                }
                if cut_at_end {
                    flags |= 0b1 << 3;
                }
                if high_dpi {
                    flags |= 0b1 << 6;
                }
                vec![0x1b, 0x69, 0x4b, flags]
            }
            RC::PrintInformation {
                media,
                quality_priority,
                recovery_on,
                no_lines,
                first_page,
            } => {
                let mut valid_flag = 0x06;
                let media_width = media.width_mm();
                let mut media_length = 0x00;
                let media_type;
                if let Some(length_mm) = media.length_mm() {
                    media_type = 0x0b;
                    media_length = length_mm;
                    valid_flag |= 0x8;
                } else {
                    media_type = 0x0a;
                }
                if quality_priority {
                    valid_flag |= 0x40;
                }
                if recovery_on {
                    valid_flag |= 0x80;
                }
                let [n8, n7, n6, n5] = no_lines.to_be_bytes();
                let first_page = u8::from(!first_page);
                vec![
                    0x1b, 0x69, 0x7a,
                    valid_flag,
                    media_type,
                    media_width,
                    media_length,
                    n5, n6, n7, n8,
                    first_page,
                    0x00,
                ]
            }
        }
    }
}

#[derive(Default)]
pub(crate) struct RasterCommands {
    commands: Vec<Vec<u8>>,
}

impl RasterCommands {
    pub fn add(&mut self, cmd: RasterCommand) {
        self.commands.push(cmd.into());
    }

    pub fn build(self) -> Vec<u8> {
        self.commands.concat()
    }

    pub fn create_preamble() -> Self {
        let mut res = Self::default();
        res.add(RasterCommand::Invalidate);
        res.add(RasterCommand::Initialize);
        res
    }
}
