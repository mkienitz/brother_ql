#[derive(Clone, Copy)]
pub struct MediaSettings {
    pub media_type: MediaType,
    pub width_dots: u32,
    pub width_mm: u8,
    pub left_margin: u32,
    pub color: bool,
}

pub const MAX_WIDTH: usize = 720;

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

#[derive(Clone, Copy)]
pub enum MediaType {
    Continuous,
    DieCut { length_dots: u32, length_mm: u8 },
}

pub enum Media {
    C12,
    C29,
    C38,
    C50,
    C54,
    C62,
    C62R,
    D17x54,
    D17x87,
    D23x23,
    D29x42,
    D29x90,
    D38x90,
    D39x48,
    D52x29,
    D54x29,
    D60x86,
    D62x29,
    D62x60,
    D62x75,
    D62x100,
    D12,
    D24,
    D58,
}
