use crate::media::{MediaSettings, MediaType};

pub(crate) enum DynamicCommandMode {
    // EscP,
    Raster,
    // PTouchTemplate,
}

pub(crate) enum ColorPower {
    LowEnergy,
    HighEnergy,
}

pub(crate) enum RasterCommand {
    Initialize,
    Invalidate,
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
    // ZeroRasterGraphics,
    Print,
    PrintWithFeed,
    SelectCompressionMode {
        tiff_compression: bool,
    },
    SpecifyPageNumber {
        cut_every: u8,
    },
    VariousMode {
        auto_cut: bool,
    },
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
}

impl From<RasterCommand> for Vec<u8> {
    fn from(value: RasterCommand) -> Self {
        use RasterCommand::*;
        match value {
            Invalidate => {
                vec![0u8; 400]
            }
            Initialize => {
                vec![0x1b, 0x40]
            }
            SpecifyMarginAmount { margin_size } => {
                let [n2, n1] = margin_size.to_be_bytes();
                vec![0x1b, 0x69, 0x64, n1, n2]
            }
            SwitchDynamicCommandMode { command_mode } => {
                use DynamicCommandMode::*;
                let m = match command_mode {
                    // EscP => 0x00,
                    Raster => 0x01,
                    // PTouchTemplate => 0x03,
                };
                vec![0x1b, 0x69, 0x61, m]
            }
            SwitchAutomaticStatusNotificationMode { notify } => {
                let n = if notify { 0x00 } else { 0x01 };
                vec![0x1b, 0x69, 0x21, n]
            }
            RasterGraphicsTransfer { mut data } => {
                let mut res = vec![0x67, 0x00, data.len() as u8];
                res.append(&mut data);
                res
            }
            TwoColorRasterGraphicsTransfer {
                mut data,
                color_power,
            } => {
                let cp = match color_power {
                    ColorPower::HighEnergy => 0x01,
                    ColorPower::LowEnergy => 0x02,
                };
                let mut res = vec![0x77, cp, data.len() as u8];
                res.append(&mut data);
                res
            }
            // ZeroRasterGraphics => {
            //     vec![0x5a]
            // }
            Print => {
                vec![0x0c]
            }
            PrintWithFeed => {
                vec![0x1a]
            }
            SelectCompressionMode { tiff_compression } => {
                let cm = if tiff_compression { 0x02 } else { 0x00 };
                vec![0x4d, cm]
            }
            SpecifyPageNumber { cut_every } => {
                vec![0x1b, 0x69, 0x41, cut_every]
            }
            VariousMode { auto_cut } => {
                let ac = if auto_cut { 0b1 << (7 - 1) } else { 0x00 };
                vec![0x1b, 0x69, 0x4d, ac]
            }
            ExpandedMode {
                two_color,
                cut_at_end,
                high_dpi,
            } => {
                let mut flags = 0x00;
                if two_color {
                    flags |= 0b1;
                }
                if cut_at_end {
                    flags |= 0b1 << 3
                }
                if high_dpi {
                    flags |= 0b1 << 6
                }
                vec![0x1b, 0x69, 0x4b, flags]
            }
            PrintInformation {
                media_settings,
                quality_priority,
                recovery_on,
                no_lines,
                first_page,
            } => {
                // Media Type and Media Length are always valid
                let mut valid_flag = 0x06;
                let media_type;
                let media_width = media_settings.width_mm;
                let mut media_length = 0x00;
                match media_settings.media_type {
                    MediaType::Continuous => {
                        media_type = 0x0a;
                    }
                    MediaType::DieCut { length_mm, .. } => {
                        media_type = 0x0b;
                        media_length = length_mm;
                        valid_flag |= 0x8;
                    }
                };
                if quality_priority {
                    valid_flag |= 0x40;
                }
                if recovery_on {
                    valid_flag |= 0x80;
                }
                let [n8, n7, n6, n5] = no_lines.to_be_bytes();
                let first_page = if first_page { 0x00 } else { 0x01 };
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
pub(crate) struct CommandBuilder {
    commands: Vec<Vec<u8>>,
}

impl CommandBuilder {
    pub fn add(&mut self, cmd: RasterCommand) {
        self.commands.push(cmd.into())
    }

    pub fn build(self) -> Vec<u8> {
        self.commands.concat()
    }
}
