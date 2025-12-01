//! Definitions for the available paper media types
#[cfg(feature = "serde")]
use serde::Deserialize;

use crate::error::StatusParsingError;

/// This enum represents the available paper types.
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, strum::EnumIter)]
pub enum Media {
    /// Continous 12mm wide roll
    C12,
    /// Continous 29mm wide roll
    C29,
    /// Continous 38mm wide roll
    C38,
    /// Continous 50mm wide roll
    C50,
    /// Continous 54mm wide roll
    C54,
    /// Continous 62mm wide roll
    C62,
    /// Continous 62mm wide roll with dual-color support (black/red)
    C62R,
    /// Die-cut 17x54mm labels
    D17x54,
    /// Die-cut 17x87mm labels
    D17x87,
    /// Die-cut 23x23mm labels
    D23x23,
    /// Die-cut 29x42mm labels
    D29x42,
    /// Die-cut 29x90mm labels
    D29x90,
    /// Die-cut 38x90mm labels
    D38x90,
    /// Die-cut 39x48mm labels
    D39x48,
    /// Die-cut 52x29mm labels
    D52x29,
    /// Die-cut 54x29mm labels
    D54x29,
    /// Die-cut 60x86mm labels
    D60x86,
    /// Die-cut 62x29mm labels
    D62x29,
    /// Die-cut 62x60mm labels
    D62x100,
    /// Die-cut 12mm circle labels
    D12,
    /// Die-cut 24mm circle labels
    D24,
    /// Die-cut 58mm circle labels
    D58,
}

/// Media type of the label roll
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum MediaType {
    /// Continuous label roll
    Continuous,
    /// Die-cut labels with
    DieCut,
}

impl TryFrom<u8> for MediaType {
    type Error = StatusParsingError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            // The documentation states 0x4a and 0x4b instead
            0x0a => Ok(Self::Continuous),
            0x0b => Ok(Self::DieCut),
            invalid => Err(StatusParsingError {
                reason: format!("invalid media type code {invalid:#x}"),
            }),
        }
    }
}

/// Length information for media
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub(crate) enum LengthInfo {
    /// Continuous media has no fixed length
    Endless,
    /// Fixed-length media (die-cut labels)
    Fixed {
        /// Length in dots
        length_dots: u32,
        /// Length in millimeters
        length_mm: u8,
    },
}

/// Physical settings and dimensions for a media type
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub(crate) struct MediaSettings {
    /// Type of label roll (continuous or die-cut)
    pub(crate) media_type: MediaType,
    /// Width in dots
    pub(crate) width_dots: u32,
    /// Width in millimeters
    pub(crate) width_mm: u8,
    /// Length information
    pub(crate) length_info: LengthInfo,
    /// Left margin in dots
    pub(crate) left_margin: u32,
    /// Whether this media supports color printing
    pub(crate) color: bool,
}

/// Helper macro for constructing `MediaSettings`
macro_rules! media_settings {
    // Internal builder
    (@build $media_type:expr, $length_info:expr, $width_mm:expr, $width_dots:expr, $left_margin:expr, $color:expr) => {
        MediaSettings {
            media_type: $media_type,
            width_dots: $width_dots,
            width_mm: $width_mm,
            length_info: $length_info,
            left_margin: $left_margin,
            color: $color,
        }
    };
    // Continuous (default: no color)
    (continuous, $width_mm:expr, $width_dots:expr, $left_margin:expr) => {
        media_settings!(@build MediaType::Continuous, LengthInfo::Endless, $width_mm, $width_dots, $left_margin, false)
    };
    // Continuous with color
    (continuous color, $width_mm:expr, $width_dots:expr, $left_margin:expr) => {
        media_settings!(@build MediaType::Continuous, LengthInfo::Endless, $width_mm, $width_dots, $left_margin, true)
    };
    // Die-cut (default: no color)
    (die_cut, $width_mm:expr, $width_dots:expr, $length_mm:expr, $length_dots:expr, $left_margin:expr) => {
        media_settings!(@build MediaType::DieCut, LengthInfo::Fixed { length_dots: $length_dots, length_mm: $length_mm }, $width_mm, $width_dots, $left_margin, false)
    };
}

impl From<Media> for MediaSettings {
    /// Create media settings for a specific media type
    fn from(value: Media) -> Self {
        match value {
            Media::C12 => media_settings!(continuous, 12, 106, 585),
            Media::C29 => media_settings!(continuous, 29, 306, 408),
            Media::C38 => media_settings!(continuous, 38, 413, 295),
            Media::C50 => media_settings!(continuous, 50, 554, 154),
            Media::C54 => media_settings!(continuous, 54, 590, 130),
            Media::C62 => media_settings!(continuous, 62, 696, 12),
            Media::C62R => media_settings!(continuous color, 62, 696, 12),
            Media::D17x54 => media_settings!(die_cut, 17, 165, 54, 566, 555),
            Media::D17x87 => media_settings!(die_cut, 17, 165, 87, 956, 555),
            Media::D23x23 => media_settings!(die_cut, 23, 236, 23, 202, 442),
            Media::D29x42 => media_settings!(die_cut, 29, 306, 42, 425, 408),
            Media::D29x90 => media_settings!(die_cut, 29, 306, 90, 991, 408),
            Media::D38x90 => media_settings!(die_cut, 38, 413, 90, 991, 295),
            Media::D39x48 => media_settings!(die_cut, 39, 425, 48, 495, 289),
            Media::D52x29 => media_settings!(die_cut, 52, 578, 29, 271, 142),
            Media::D54x29 => media_settings!(die_cut, 54, 602, 29, 271, 59),
            Media::D60x86 => media_settings!(die_cut, 60, 672, 86, 954, 24),
            Media::D62x29 => media_settings!(die_cut, 62, 696, 29, 271, 12),
            Media::D62x100 => media_settings!(die_cut, 62, 696, 100, 1109, 12),
            Media::D12 => media_settings!(die_cut, 24, 94, 12, 94, 513),
            Media::D24 => media_settings!(die_cut, 24, 236, 24, 236, 442),
            Media::D58 => media_settings!(die_cut, 58, 618, 58, 618, 51),
        }
    }
}
