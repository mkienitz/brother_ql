use crate::{
    error::StatusParsingError,
    media::{LengthInfo, MediaSettings},
};

pub(crate) enum DynamicCommandMode {
    // EscP,
    Raster,
    // PTouchTemplate,
}

pub(crate) enum ColorPower {
    LowEnergy,
    HighEnergy,
}

#[derive(Debug, Clone, Copy)]
pub struct VariousModeSettings {
    pub auto_cut: bool,
}

impl TryFrom<u8> for VariousModeSettings {
    type Error = StatusParsingError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let ac = match value {
            0x40 => Ok(true),
            0x00 => Ok(false),
            _ => Err(StatusParsingError {
                reason: "various mode data has unused bits set".to_string(),
            }),
        }?;
        Ok(VariousModeSettings { auto_cut: ac })
    }
}

pub(crate) enum RasterCommand {
    // Initialization Commands
    Invalidate,
    StatusInformationRequest,
    Initialize,
    // Control Codes
    SpecifyMarginAmount {
        margin_size: u16,
    },
    SwitchDynamicCommandMode {
        command_mode: DynamicCommandMode,
    },
    SwitchAutomaticStatusNotificationMode {
        notify: bool,
    },
    RasterGraphicsTransfer {
        data: Vec<u8>,
    },
    TwoColorRasterGraphicsTransfer {
        data: Vec<u8>,
        color_power: ColorPower,
    },
    SelectCompressionMode {
        tiff_compression: bool,
    },
    SpecifyPageNumber {
        cut_every: u8,
    },
    VariousMode(VariousModeSettings),
    ExpandedMode {
        two_color: bool,
        cut_at_end: bool,
        high_dpi: bool,
    },
    PrintInformation {
        media_settings: MediaSettings,
        quality_priority: bool,
        recovery_on: bool,
        no_lines: u32,
        first_page: bool,
    },
    // Print Commands
    Print,
    PrintWithFeed,
}

impl From<RasterCommand> for Vec<u8> {
    #[allow(clippy::too_many_lines)]
    fn from(value: RasterCommand) -> Self {
        use RasterCommand as RC;
        match value {
            RC::Invalidate => {
                vec![0u8; 400]
            }
            RC::StatusInformationRequest => {
                vec![0x1b, 0x69, 0x53]
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
                    // EscP => 0x00,
                    DynamicCommandMode::Raster => 0x01,
                    // PTouchTemplate => 0x03,
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
            RC::TwoColorRasterGraphicsTransfer {
                mut data,
                color_power,
            } => {
                let cp = match color_power {
                    ColorPower::HighEnergy => 0x01,
                    ColorPower::LowEnergy => 0x02,
                };
                #[allow(clippy::cast_possible_truncation)]
                let mut res = vec![0x77, cp, data.len() as u8];
                res.append(&mut data);
                res
            }
            // NOTE: According to specification, the QL800 does not support this command
            // Maybe investigate whether the whole series supports it or not.
            // RC::ZeroRasterGraphics => {
            //     vec![0x5a]
            // }
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
            RC::ExpandedMode {
                two_color,
                cut_at_end,
                high_dpi,
            } => {
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
                media_settings,
                quality_priority,
                recovery_on,
                no_lines,
                first_page,
            } => {
                // Media Type and Media Length are always valid
                let mut valid_flag = 0x06;
                let media_width = media_settings.width_mm;
                let mut media_length = 0x00;
                let media_type;
                match media_settings.length_info {
                    LengthInfo::Endless => {
                        media_type = 0x0a;
                    }
                    LengthInfo::Fixed { length_mm, .. } => {
                        media_type = 0x0b;
                        media_length = length_mm;
                        valid_flag |= 0x8;
                    }
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
                    0x1b,
                    0x69,
                    0x7a,
                    valid_flag,
                    media_type,
                    media_width,
                    media_length,
                    n5,
                    n6,
                    n7,
                    n8,
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
