//! Media type definitions and settings
//!
//! This module defines all supported label media types for Brother QL printers.
//! Media types include both continuous rolls and die-cut labels in various sizes.
//!
//! # Media Naming Convention
//!
//! - **C** prefix = Continuous roll (e.g., [`Media::C62`] = 62mm continuous)
//! - **D** prefix = Die-cut labels (e.g., [`Media::D17x54`] = 17mm × 54mm labels)
//! - **R** suffix = Red/black two-color support (e.g., [`Media::C62R`])
//!
//! # Continuous vs. Die-Cut
//!
//! **Continuous rolls** can be cut to any length by the printer. The image height
//! can be any size, and the printer will feed the appropriate amount of media.
//!
//! **Die-cut labels** are pre-cut to specific dimensions. Your image must match
//! the exact label dimensions, or you'll get a dimension mismatch error.
//!
//! # Image Dimensions
//!
//! The required image dimensions in pixels are documented in [`Media`]
//!
//! # Color Printing
//!
//! Only [`Media::C62R`] currently supports two-color (black/red) printing.
//! All other media types support black-only printing.
//!
//! See [`PrintJob::from_image`](crate::printjob::PrintJob::from_image) for details.

use crate::error::StatusParsingError;

/// Type of label media
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LabelType {
    /// Continuous roll media (cut to any length)
    Continuous,
    /// Die-cut pre-sized labels
    DieCut,
}

impl TryFrom<u8> for LabelType {
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

macro_rules! define_media {
    // Optional literal → Option
    (@opt $val:literal) => { Some($val) };
    (@opt) => { None };

    // Bullet list formatting
    (@products [$($product:literal),+]) => {
        concat!($("\n- ", $product),+)
    };

    // Continuous tape
    (@ty_doc Continuous, , , $wmm:literal, $wpx:literal) => {
        concat!(
            "Continuous tape, ", $wmm, "mm (", $wpx, "px) wide"
        )
    };

    // DieCut labels
    (@ty_doc DieCut, $lmm:literal, $lpx:literal, $wmm:literal, $wpx:literal) => {
        concat!(
            "Die-cut labels, WxH ", $wmm, "x", $lmm, "mm (", $wpx, "x", $lpx, "px)"
        )
    };

    (
        $(
            $name:ident {
                products: [$($product:literal),+ $(,)?],
                label_type: $label_type:ident,
                width_mm: $width_mm:literal,
                width_dots: $width_dots:literal,
                left_margin: $left_margin:literal,
                supports_color: $supports_color:literal,
                $( length_mm: $length_mm:literal, length_dots: $length_dots:literal, )?
            }
        ),+ $(,)?
    ) => {
        /// Available media types for Brother QL printers
        #[derive(Copy, PartialEq, Clone, Debug, strum::EnumIter, strum::Display)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        pub enum Media {
            $(
                #[doc = concat!(
                    define_media!(
                        @ty_doc
                        $label_type,
                        $( $length_mm )?,
                        $( $length_dots )?,
                        $width_mm,
                        $width_dots
                    ),
                    "\n\n**Compatible products:**",
                    define_media!(@products [$($product),+])
                )]
                $name,
            )+
        }

        impl Media {
            /// Returns the label type (`Continuous` or `DieCut`)
            pub(crate) const fn label_type(self) -> LabelType {
                match self { $( Media::$name => LabelType::$label_type ),+ }
            }
            /// Returns the media width in millimeters
            pub(crate) const fn width_mm(self) -> u8 {
                match self { $( Media::$name => $width_mm ),+ }
            }
            /// Returns the media width in dots (at 300 DPI)
            pub(crate) const fn width_dots(self) -> u32 {
                match self { $( Media::$name => $width_dots ),+ }
            }
            /// Returns the left margin in dots
            pub(crate) const fn left_margin(self) -> u32 {
                match self { $( Media::$name => $left_margin ),+ }
            }
            /// Returns whether this media supports red/black two-color printing
            pub(crate) const fn supports_color(self) -> bool {
                match self { $( Media::$name => $supports_color ),+ }
            }
            /// Returns the label length in millimeters (`None` for continuous media)
            pub(crate) const fn length_mm(self) -> Option<u8> {
                match self { $( Media::$name => define_media!(@opt $( $length_mm )? ) ),+ }
            }
            /// Returns the label length in dots (None for continuous media)
            pub(crate) const fn length_dots(self) -> Option<u32> {
                match self { $( Media::$name => define_media!(@opt $( $length_dots )? ) ),+ }
            }
        }
    };
}

define_media! {
    C12 {
        products: ["DK-22214 Tape"],
        label_type: Continuous,
        width_mm: 12,
        width_dots: 106,
        left_margin: 585,
        supports_color: false,
    },
    C29 {
        products: ["DK-22210 Tape", "DK-22211 Film Tape"],
        label_type: Continuous,
        width_mm: 29,
        width_dots: 306,
        left_margin: 408,
        supports_color: false,
    },
    C38 {
        products: ["DK-22225 Tape"],
        label_type: Continuous,
        width_mm: 38,
        width_dots: 413,
        left_margin: 295,
        supports_color: false,
    },
    C50 {
        products: ["DK-22223 Tape"],
        label_type: Continuous,
        width_mm: 50,
        width_dots: 554,
        left_margin: 154,
        supports_color: false,
    },
    C54 {
        products: ["DK-N55224 Non-Adhesive Tape"],
        label_type: Continuous,
        width_mm: 54,
        width_dots: 590,
        left_margin: 130,
        supports_color: false,
    },
    C62 {
        products: ["DK-22205 Tape", "DK-22212 Film Tape", "DK-22606 Yellow Film Tape", "DDK-22213 Transparent Tape", "DK-44205 Removable Tape", "DK-44605 Yellow Removable Tape"],
        label_type: Continuous,
        width_mm: 62,
        width_dots: 696,
        left_margin: 12,
        supports_color: false,
    },
    C62R {
        products: ["DK-22251 Red/Black Tape"],
        label_type: Continuous,
        width_mm: 62,
        width_dots: 696,
        left_margin: 12,
        supports_color: true,
    },
    D17x54 {
        products: ["DK-11204 Labels"],
        label_type: DieCut,
        width_mm: 17,
        width_dots: 165,
        left_margin: 555,
        supports_color: false,
        length_mm: 54,
        length_dots: 566,
    },
    D17x87 {
        products: ["DK-11203 File Folder Labels"],
        label_type: DieCut,
        width_mm: 17,
        width_dots: 165,
        left_margin: 555,
        supports_color: false,
        length_mm: 87,
        length_dots: 912,
    },
    D23x23 {
        products: ["DK-11221 Square Labels"],
        label_type: DieCut,
        width_mm: 23,
        width_dots: 236,
        left_margin: 442,
        supports_color: false,
        length_mm: 23,
        length_dots: 236,
    },
    D29x42 {
        products: ["DK-11215 Labels (⚠️ no official data)"],
        label_type: DieCut,
        width_mm: 29,
        width_dots: 306,
        left_margin: 408,
        supports_color: false,
        length_mm: 42,
        length_dots: 442,
    },
    D29x90 {
        products: ["DK-11201 Standard Address Labels"],
        label_type: DieCut,
        width_mm: 29,
        width_dots: 306,
        left_margin: 408,
        supports_color: false,
        length_mm: 90,
        length_dots: 944,
    },
    D38x90 {
        products: ["DK-11208 Large Address Labels"],
        label_type: DieCut,
        width_mm: 38,
        width_dots: 413,
        left_margin: 295,
        supports_color: false,
        length_mm: 90,
        length_dots: 944,
    },
    D39x48 {
        products: ["DK-11220 Labels (⚠️ no official data)"],
        label_type: DieCut,
        width_mm: 39,
        width_dots: 425,
        left_margin: 289,
        supports_color: false,
        length_mm: 48,
        length_dots: 512,
    },
    D52x29 {
        products: ["DK-11226 Labels (⚠️ no official data)"],
        label_type: DieCut,
        width_mm: 52,
        width_dots: 578,
        left_margin: 142,
        supports_color: false,
        length_mm: 29,
        length_dots: 318,
    },
    D54x29 {
        products: ["DK-3235 Removable"],
        label_type: DieCut,
        width_mm: 54,
        width_dots: 602,
        left_margin: 59,
        supports_color: false,
        length_mm: 29,
        length_dots: 318,
    },
    D60x86 {
        products: ["DK-11234 Name Badge Labels"],
        label_type: DieCut,
        width_mm: 60,
        width_dots: 672,
        left_margin: 24,
        supports_color: false,
        length_mm: 86,
        length_dots: 902,
    },
    D62x29 {
        products: ["DK-11209 Small Address Labels"],
        label_type: DieCut,
        width_mm: 62,
        width_dots: 696,
        left_margin: 12,
        supports_color: false,
        length_mm: 29,
        length_dots: 318,
    },
    D62x100 {
        products: ["DK-11202 Shipping Labels"],
        label_type: DieCut,
        width_mm: 62,
        width_dots: 696,
        left_margin: 12,
        supports_color: false,
        length_mm: 100,
        length_dots: 1104,
    },
    D12 {
        products: ["DK-11219 Round Labels"],
        label_type: DieCut,
        width_mm: 12,
        width_dots: 94,
        left_margin: 513,
        supports_color: false,
        length_mm: 12,
        length_dots: 94,
    },
    D24 {
        products: ["DK-11218 Round Labels"],
        label_type: DieCut,
        width_mm: 24,
        width_dots: 236,
        left_margin: 442,
        supports_color: false,
        length_mm: 24,
        length_dots: 236,
    },
    D58 {
        products: ["DK-11207 CD/DVD Labels"],
        label_type: DieCut,
        width_mm: 58,
        width_dots: 618,
        left_margin: 51,
        supports_color: false,
        length_mm: 58,
        length_dots: 630,
    },
}
