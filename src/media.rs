//! Definitions for the available paper media types
#[cfg(feature = "serde")]
use serde::Deserialize;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub(crate) struct MediaSettings {
    pub media_type: MediaType,
    pub width_dots: u32,
    pub width_mm: u8,
    pub left_margin: u32,
    pub color: bool,
}

impl MediaSettings {
    pub fn new(media: &Media) -> Self {
        match media {
            Media::C62 => Self {
                media_type: MediaType::Continuous,
                width_dots: 696,
                width_mm: 62,
                left_margin: 12,
                color: false,
            },
            Media::C62R => Self {
                media_type: MediaType::Continuous,
                width_dots: 696,
                width_mm: 62,
                left_margin: 12,
                color: true,
            },
            Media::D24 => Self {
                media_type: MediaType::DieCut {
                    length_dots: 236,
                    length_mm: 24,
                },
                width_dots: 236,
                width_mm: 24,
                left_margin: 442,
                color: false,
            },
            _ => todo!(),
        }
    }
}

/// This enum represents the basic two media types:
/// * continuous label rolls
/// * die-cut labels
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub(crate) enum MediaType {
    Continuous,
    DieCut { length_dots: u32, length_mm: u8 },
}

/// This enum represents the available paper types.
///
/// **Important note:**
/// Currently, only [C62][Media::C62], [C62R][Media::C62R] and [D24][Media::D24] are supported.
#[cfg_attr(feature = "serde", derive(Deserialize))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
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
    D62x60,
    /// Die-cut 62x75mm labels
    D62x75,
    /// Die-cut 62x10mm labels
    D62x100,
    /// Die-cut 12mm circle labels
    D12,
    /// Die-cut 24mm circle labels
    D24,
    /// Die-cut 58mm circle labels
    D58,
}
