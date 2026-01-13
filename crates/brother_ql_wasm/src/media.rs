//! Media type definitions for Brother QL printers

use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

/// Type of label media
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LabelType {
    /// Continuous roll media (cut to any length)
    Continuous,
    /// Die-cut pre-sized labels
    DieCut,
}

/// Available media types for Brother QL printers
#[derive(Copy, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Media {
    /// Continuous tape, 12mm (106px) wide
    C12,
    /// Continuous tape, 29mm (306px) wide
    C29,
    /// Continuous tape, 38mm (413px) wide
    C38,
    /// Continuous tape, 50mm (554px) wide
    C50,
    /// Continuous tape, 54mm (590px) wide
    C54,
    /// Continuous tape, 62mm (696px) wide
    C62,
    /// Continuous tape, 62mm (696px) wide, Red/Black
    C62R,
    /// Die-cut labels, 17x54mm (165x566px)
    D17x54,
    /// Die-cut labels, 17x87mm (165x912px)
    D17x87,
    /// Die-cut labels, 23x23mm (236x236px)
    D23x23,
    /// Die-cut labels, 29x42mm (306x442px)
    D29x42,
    /// Die-cut labels, 29x90mm (306x944px)
    D29x90,
    /// Die-cut labels, 38x90mm (413x944px)
    D38x90,
    /// Die-cut labels, 39x48mm (425x512px)
    D39x48,
    /// Die-cut labels, 52x29mm (578x318px)
    D52x29,
    /// Die-cut labels, 54x29mm (602x318px)
    D54x29,
    /// Die-cut labels, 60x86mm (672x902px)
    D60x86,
    /// Die-cut labels, 62x29mm (696x318px)
    D62x29,
    /// Die-cut labels, 62x100mm (696x1104px)
    D62x100,
    /// Round labels, 12mm diameter (94px)
    D12,
    /// Round labels, 24mm diameter (236px)
    D24,
    /// CD/DVD labels, 58mm (618x630px)
    D58,
}

impl Media {
    /// Returns the label type (Continuous or DieCut)
    pub fn label_type(self) -> LabelType {
        match self {
            Media::C12 | Media::C29 | Media::C38 | Media::C50 | Media::C54 | Media::C62 | Media::C62R => LabelType::Continuous,
            _ => LabelType::DieCut,
        }
    }

    /// Returns the media width in millimeters
    pub fn width_mm(self) -> u8 {
        match self {
            Media::C12 => 12,
            Media::C29 => 29,
            Media::C38 => 38,
            Media::C50 => 50,
            Media::C54 => 54,
            Media::C62 | Media::C62R => 62,
            Media::D17x54 | Media::D17x87 => 17,
            Media::D23x23 => 23,
            Media::D29x42 | Media::D29x90 => 29,
            Media::D38x90 => 38,
            Media::D39x48 => 39,
            Media::D52x29 => 52,
            Media::D54x29 => 54,
            Media::D60x86 => 60,
            Media::D62x29 | Media::D62x100 => 62,
            Media::D12 => 12,
            Media::D24 => 24,
            Media::D58 => 58,
        }
    }

    /// Returns the media width in dots (pixels at 300 DPI)
    pub fn width_dots(self) -> u32 {
        match self {
            Media::C12 => 106,
            Media::C29 => 306,
            Media::C38 => 413,
            Media::C50 => 554,
            Media::C54 => 590,
            Media::C62 | Media::C62R => 696,
            Media::D17x54 | Media::D17x87 => 165,
            Media::D23x23 => 236,
            Media::D29x42 | Media::D29x90 => 306,
            Media::D38x90 => 413,
            Media::D39x48 => 425,
            Media::D52x29 => 578,
            Media::D54x29 => 602,
            Media::D60x86 => 672,
            Media::D62x29 | Media::D62x100 => 696,
            Media::D12 => 94,
            Media::D24 => 236,
            Media::D58 => 618,
        }
    }

    /// Returns the left margin in dots
    pub(crate) fn left_margin(self) -> u32 {
        match self {
            Media::C12 => 585,
            Media::C29 => 408,
            Media::C38 => 295,
            Media::C50 => 154,
            Media::C54 => 130,
            Media::C62 | Media::C62R => 12,
            Media::D17x54 | Media::D17x87 => 555,
            Media::D23x23 => 442,
            Media::D29x42 | Media::D29x90 => 408,
            Media::D38x90 => 295,
            Media::D39x48 => 289,
            Media::D52x29 => 142,
            Media::D54x29 => 59,
            Media::D60x86 => 24,
            Media::D62x29 | Media::D62x100 => 12,
            Media::D12 => 513,
            Media::D24 => 442,
            Media::D58 => 51,
        }
    }

    /// Returns whether this media supports red/black two-color printing
    pub fn supports_color(self) -> bool {
        matches!(self, Media::C62R)
    }

    /// Returns the label length in millimeters (None for continuous media)
    pub fn length_mm(self) -> Option<u8> {
        match self {
            Media::C12 | Media::C29 | Media::C38 | Media::C50 | Media::C54 | Media::C62 | Media::C62R => None,
            Media::D17x54 => Some(54),
            Media::D17x87 => Some(87),
            Media::D23x23 => Some(23),
            Media::D29x42 => Some(42),
            Media::D29x90 => Some(90),
            Media::D38x90 => Some(90),
            Media::D39x48 => Some(48),
            Media::D52x29 => Some(29),
            Media::D54x29 => Some(29),
            Media::D60x86 => Some(86),
            Media::D62x29 => Some(29),
            Media::D62x100 => Some(100),
            Media::D12 => Some(12),
            Media::D24 => Some(24),
            Media::D58 => Some(58),
        }
    }

    /// Returns the label length in dots (None for continuous media)
    pub fn length_dots(self) -> Option<u32> {
        match self {
            Media::C12 | Media::C29 | Media::C38 | Media::C50 | Media::C54 | Media::C62 | Media::C62R => None,
            Media::D17x54 => Some(566),
            Media::D17x87 => Some(912),
            Media::D23x23 => Some(236),
            Media::D29x42 => Some(442),
            Media::D29x90 => Some(944),
            Media::D38x90 => Some(944),
            Media::D39x48 => Some(512),
            Media::D52x29 => Some(318),
            Media::D54x29 => Some(318),
            Media::D60x86 => Some(902),
            Media::D62x29 => Some(318),
            Media::D62x100 => Some(1104),
            Media::D12 => Some(94),
            Media::D24 => Some(236),
            Media::D58 => Some(630),
        }
    }

    /// Get a human-readable name for this media type
    pub fn display_name(self) -> String {
        match self {
            Media::C12 => "12mm Continuous".to_string(),
            Media::C29 => "29mm Continuous".to_string(),
            Media::C38 => "38mm Continuous".to_string(),
            Media::C50 => "50mm Continuous".to_string(),
            Media::C54 => "54mm Continuous".to_string(),
            Media::C62 => "62mm Continuous".to_string(),
            Media::C62R => "62mm Continuous (Red/Black)".to_string(),
            Media::D17x54 => "17x54mm Die-cut".to_string(),
            Media::D17x87 => "17x87mm Die-cut".to_string(),
            Media::D23x23 => "23x23mm Square".to_string(),
            Media::D29x42 => "29x42mm Die-cut".to_string(),
            Media::D29x90 => "29x90mm Address".to_string(),
            Media::D38x90 => "38x90mm Large Address".to_string(),
            Media::D39x48 => "39x48mm Die-cut".to_string(),
            Media::D52x29 => "52x29mm Die-cut".to_string(),
            Media::D54x29 => "54x29mm Die-cut".to_string(),
            Media::D60x86 => "60x86mm Name Badge".to_string(),
            Media::D62x29 => "62x29mm Small Address".to_string(),
            Media::D62x100 => "62x100mm Shipping".to_string(),
            Media::D12 => "12mm Round".to_string(),
            Media::D24 => "24mm Round".to_string(),
            Media::D58 => "58mm CD/DVD".to_string(),
        }
    }

    /// Parse a media type from a string name
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "C12" => Some(Media::C12),
            "C29" => Some(Media::C29),
            "C38" => Some(Media::C38),
            "C50" => Some(Media::C50),
            "C54" => Some(Media::C54),
            "C62" => Some(Media::C62),
            "C62R" => Some(Media::C62R),
            "D17x54" => Some(Media::D17x54),
            "D17x87" => Some(Media::D17x87),
            "D23x23" => Some(Media::D23x23),
            "D29x42" => Some(Media::D29x42),
            "D29x90" => Some(Media::D29x90),
            "D38x90" => Some(Media::D38x90),
            "D39x48" => Some(Media::D39x48),
            "D52x29" => Some(Media::D52x29),
            "D54x29" => Some(Media::D54x29),
            "D60x86" => Some(Media::D60x86),
            "D62x29" => Some(Media::D62x29),
            "D62x100" => Some(Media::D62x100),
            "D12" => Some(Media::D12),
            "D24" => Some(Media::D24),
            "D58" => Some(Media::D58),
            _ => None,
        }
    }
}

/// Get all available media type names
#[wasm_bindgen(js_name = getAllMediaTypeNames)]
pub fn get_all_media_type_names() -> Vec<String> {
    vec![
        "C12".to_string(), "C29".to_string(), "C38".to_string(), "C50".to_string(), 
        "C54".to_string(), "C62".to_string(), "C62R".to_string(),
        "D17x54".to_string(), "D17x87".to_string(), "D23x23".to_string(), 
        "D29x42".to_string(), "D29x90".to_string(), "D38x90".to_string(), 
        "D39x48".to_string(), "D52x29".to_string(), "D54x29".to_string(), 
        "D60x86".to_string(), "D62x29".to_string(), "D62x100".to_string(), 
        "D12".to_string(), "D24".to_string(), "D58".to_string(),
    ]
}

/// Get media info as JSON for JavaScript consumption
#[wasm_bindgen(js_name = getMediaInfo)]
pub fn get_media_info(media_name: &str) -> JsValue {
    let media = match Media::from_str(media_name) {
        Some(m) => m,
        None => return JsValue::NULL,
    };
    let label_type = match media.label_type() {
        LabelType::Continuous => "Continuous",
        LabelType::DieCut => "DieCut",
    };
    let info = serde_json::json!({
        "name": media_name,
        "displayName": media.display_name(),
        "labelType": label_type,
        "widthMm": media.width_mm(),
        "widthDots": media.width_dots(),
        "lengthMm": media.length_mm(),
        "lengthDots": media.length_dots(),
        "supportsColor": media.supports_color(),
    });
    serde_wasm_bindgen::to_value(&info).unwrap()
}
