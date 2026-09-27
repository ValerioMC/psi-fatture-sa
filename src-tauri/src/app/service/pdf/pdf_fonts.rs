use std::sync::OnceLock;

use super::{Face, LoadedFace, TextStyle};

const GEIST: &[u8] = include_bytes!("../../../../resources/fonts/Geist-Variable.ttf");
const NEWSREADER: &[u8] = include_bytes!("../../../../resources/fonts/Newsreader-Variable.ttf");

/// The faces of the invoice, bundled in the binary so a PDF looks the same on
/// every computer and needs nothing installed. Parsed once per run.
pub struct PdfFonts {
    /// In `Face::ALL` order, so a face's discriminant is its index.
    faces: Vec<LoadedFace>,
}

impl PdfFonts {
    pub fn shared() -> Result<&'static PdfFonts, String> {
        static FONTS: OnceLock<Result<PdfFonts, String>> = OnceLock::new();
        FONTS
            .get_or_init(PdfFonts::load)
            .as_ref()
            .map_err(Clone::clone)
    }

    fn load() -> Result<PdfFonts, String> {
        let faces = Face::ALL
            .iter()
            .map(|face| match face {
                Face::Regular => LoadedFace::load(GEIST, &[("wght", 400.0)]),
                Face::Medium => LoadedFace::load(GEIST, &[("wght", 500.0)]),
                Face::SemiBold => LoadedFace::load(GEIST, &[("wght", 600.0)]),
                Face::Bold => LoadedFace::load(GEIST, &[("wght", 700.0)]),
                Face::Display => LoadedFace::load(NEWSREADER, &[("wght", 600.0), ("opsz", 24.0)]),
                Face::DisplayLight => {
                    LoadedFace::load(NEWSREADER, &[("wght", 400.0), ("opsz", 16.0)])
                }
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(PdfFonts { faces })
    }

    pub fn face(&self, face: Face) -> &LoadedFace {
        &self.faces[face as usize]
    }

    pub fn width(&self, style: &TextStyle, text: &str) -> f32 {
        self.face(style.face)
            .width(text, style.size, style.tracking)
    }
}
