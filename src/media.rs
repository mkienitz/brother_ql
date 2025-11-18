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
            Media::C12 => Self {
                media_type: MediaType::Continuous,
                width_dots: 106,
                width_mm: 12,
                left_margin: 585,
                color: false,
            },
            Media::C29 => Self {
                media_type: MediaType::Continuous,
                width_dots: 306,
                width_mm: 29,
                left_margin: 408,
                color: false,
            },
            Media::C38 => Self {
                media_type: MediaType::Continuous,
                width_dots: 413,
                width_mm: 38,
                left_margin: 295,
                color: false,
            },
            Media::C50 => Self {
                media_type: MediaType::Continuous,
                width_dots: 554,
                width_mm: 50,
                left_margin: 154,
                color: false,
            },
            Media::C54 => Self {
                media_type: MediaType::Continuous,
                width_dots: 590,
                width_mm: 54,
                left_margin: 130,
                color: false,
            },
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
            Media::D17x54 => Self {
                media_type: MediaType::DieCut {
                    length_dots: 566,
                    length_mm: 54,
                },
                width_dots: 165,
                width_mm: 17,
                left_margin: 555,
                color: false,
            },
            Media::D17x87 => Self {
                media_type: MediaType::DieCut {
                    length_dots: 956,
                    length_mm: 87,
                },
                width_dots: 165,
                width_mm: 17,
                left_margin: 555,
                color: false,
            },
            Media::D23x23 => Self {
                media_type: MediaType::DieCut {
                    length_dots: 202,
                    length_mm: 23,
                },
                width_dots: 236,
                width_mm: 23,
                left_margin: 442,
                color: false,
            },
            Media::D29x42 => Self {
                media_type: MediaType::DieCut {
                    length_dots: 425,
                    length_mm: 42,
                },
                width_dots: 306,
                width_mm: 29,
                left_margin: 408,
                color: false,
            },
            Media::D29x90 => Self {
                media_type: MediaType::DieCut {
                    length_dots: 991,
                    length_mm: 90,
                },
                width_dots: 306,
                width_mm: 29,
                left_margin: 408,
                color: false,
            },
            Media::D38x90 => Self {
                media_type: MediaType::DieCut {
                    length_dots: 991,
                    length_mm: 90,
                },
                width_dots: 413,
                width_mm: 38,
                left_margin: 295,
                color: false,
            },
            Media::D39x48 => Self {
                media_type: MediaType::DieCut {
                    length_dots: 495,
                    length_mm: 48,
                },
                width_dots: 425,
                width_mm: 39,
                left_margin: 289,
                color: false,
            },
            Media::D52x29 => Self {
                media_type: MediaType::DieCut {
                    length_dots: 271,
                    length_mm: 29,
                },
                width_dots: 578,
                width_mm: 52,
                left_margin: 142,
                color: false,
            },
            Media::D54x29 => Self {
                media_type: MediaType::DieCut {
                    length_dots: 271,
                    length_mm: 29,
                },
                width_dots: 602,
                width_mm: 54,
                left_margin: 59,
                color: false,
            },
            Media::D60x86 => Self {
                media_type: MediaType::DieCut {
                    length_dots: 954,
                    length_mm: 86,
                },
                width_dots: 672,
                width_mm: 60,
                left_margin: 24,
                color: false,
            },
            Media::D62x29 => Self {
                media_type: MediaType::DieCut {
                    length_dots: 271,
                    length_mm: 29,
                },
                width_dots: 696,
                width_mm: 62,
                left_margin: 12,
                color: false,
            },
            Media::D62x100 => Self {
                media_type: MediaType::DieCut {
                    length_dots: 1109,
                    length_mm: 100,
                },
                width_dots: 696,
                width_mm: 62,
                left_margin: 12,
                color: false,
            },
            Media::D12 => Self {
                media_type: MediaType::DieCut {
                    length_dots: 94,
                    length_mm: 12,
                },
                width_dots: 94,
                width_mm: 24,
                left_margin: 513,
                color: false,
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
            Media::D58 => Self {
                media_type: MediaType::DieCut {
                    length_dots: 618,
                    length_mm: 58,
                },
                width_dots: 618,
                width_mm: 58,
                left_margin: 51,
                color: false,
            },
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
    D62x100,
    /// Die-cut 12mm circle labels
    D12,
    /// Die-cut 24mm circle labels
    D24,
    /// Die-cut 58mm circle labels
    D58,
}
