use krilla::text::{Font, GlyphId, KrillaGlyph, Tag};
use skrifa::instance::{Location, LocationRef, Size};
use skrifa::{FontRef, MetadataProvider};

/// One typeface at one point of its variation axes, able both to measure text and to hand
/// krilla the glyphs to draw. Measuring and drawing read the same advances, so a
/// right-aligned figure lands exactly where layout put it.
pub struct LoadedFace {
    font: Font,
    font_ref: FontRef<'static>,
    location: Location,
    units_per_em: f32,
}

impl LoadedFace {
    pub fn load(data: &'static [u8], axes: &[(&str, f32)]) -> Result<Self, String> {
        let font_ref = FontRef::new(data).map_err(|e| format!("Carattere non leggibile: {e}"))?;
        let location = font_ref.axes().location(axes.iter().copied());
        let units_per_em = font_ref
            .metrics(Size::unscaled(), LocationRef::default())
            .units_per_em as f32;
        let coords: Vec<(Tag, f32)> = axes
            .iter()
            .filter_map(|(tag, value)| Tag::try_from_str(tag).map(|tag| (tag, *value)))
            .collect();
        let font = Font::new_variable(data.into(), 0, &coords)
            .ok_or("Carattere non utilizzabile per il PDF")?;
        Ok(LoadedFace {
            font,
            font_ref,
            location,
            units_per_em,
        })
    }

    pub fn font(&self) -> Font {
        self.font.clone()
    }

    /// Width in points; tracking counts between glyphs, not after the last one.
    pub fn width(&self, text: &str, size: f32, tracking: f32) -> f32 {
        let glyphs = self.glyphs(text, 0.0);
        let advances: f32 = glyphs.iter().map(|glyph| glyph.x_advance).sum();
        let gaps = glyphs.len().saturating_sub(1) as f32;
        advances * size + tracking * gaps
    }

    /// One glyph per character, advances normalised to the em as krilla expects.
    /// A character the face lacks is drawn as "?" rather than as an empty box.
    pub fn glyphs(&self, text: &str, tracking_em: f32) -> Vec<KrillaGlyph> {
        let charmap = self.font_ref.charmap();
        let metrics = self
            .font_ref
            .glyph_metrics(Size::unscaled(), &self.location);
        let fallback = charmap.map('?').unwrap_or_default();
        text.char_indices()
            .map(|(start, character)| {
                let glyph = charmap.map(character).unwrap_or(fallback);
                let advance = metrics.advance_width(glyph).unwrap_or_default() / self.units_per_em;
                KrillaGlyph::new(
                    GlyphId::new(glyph.to_u32()),
                    advance + tracking_em,
                    0.0,
                    0.0,
                    0.0,
                    start..start + character.len_utf8(),
                    None,
                )
            })
            .collect()
    }

    /// Whether every character of the text has its own glyph in this face.
    #[cfg(test)]
    pub fn covers(&self, text: &str) -> bool {
        let charmap = self.font_ref.charmap();
        text.chars()
            .filter(|character| !character.is_whitespace())
            .all(|character| charmap.map(character).is_some())
    }
}
