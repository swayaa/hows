//! Markenschrift für den PDF-Export: Glyphen, Breiten und eine verkleinerte
//! TrueType-Datei mit nur den benutzten Glyphen.
//!
//! Die TTF-Dateien unter `fonts/` stehen unter der SIL Open Font License 1.1
//! (`fonts/OFL.txt`). Die Teilmenge behält die Glyph-IDs der Originaldatei
//! und leert nur die Umrisse unbenutzter Glyphen; so passen Breiten,
//! `ToUnicode` und Inhaltsströme ohne Umnummerierung zusammen.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use ttf_parser::{name_id, GlyphId};

const REGULAR_TTF: &[u8] = include_bytes!("../fonts/AtkinsonHyperlegibleNext-Regular.ttf");
const BOLD_TTF: &[u8] = include_bytes!("../fonts/AtkinsonHyperlegibleNext-Bold.ttf");

/// Ersatz für Zeichen, die der Schrift fehlen, bevor `?` greift.
const FALLBACKS: &[(char, &str)] = &[('→', "->"), ('←', "<-"), ('…', "...")];

/// Eine geladene TrueType-Schrift.
pub(crate) struct Font {
    data: &'static [u8],
    face: ttf_parser::Face<'static>,
    postscript_name: String,
}

/// Eintrag `id` der Namenstabelle, leer wenn er fehlt.
fn name(face: &ttf_parser::Face<'_>, id: u16) -> String {
    face.names()
        .into_iter()
        .filter(|name| name.name_id == id)
        .find_map(|name| name.to_string())
        .unwrap_or_default()
}

/// Eine Glyphe im Text und das Zeichen, für das sie steht.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Glyph {
    pub id: u16,
    pub text: char,
}

impl Font {
    fn load(data: &'static [u8]) -> Font {
        let face = ttf_parser::Face::parse(data, 0).expect("bundled font parses");
        let postscript_name = name(&face, name_id::POST_SCRIPT_NAME);
        Font {
            data,
            face,
            postscript_name,
        }
    }

    /// Normaler Schnitt.
    pub fn regular() -> &'static Font {
        static FONT: OnceLock<Font> = OnceLock::new();
        FONT.get_or_init(|| Font::load(REGULAR_TTF))
    }

    /// Fetter Schnitt.
    pub fn bold() -> &'static Font {
        static FONT: OnceLock<Font> = OnceLock::new();
        FONT.get_or_init(|| Font::load(BOLD_TTF))
    }

    /// PostScript-Name, etwa `AtkinsonHyperlegibleNext-Regular`.
    pub fn postscript_name(&self) -> &str {
        &self.postscript_name
    }

    /// Glyphe eines Zeichens, falls die Schrift es enthält.
    pub fn glyph_id(&self, c: char) -> Option<u16> {
        self.face.glyph_index(c).map(|glyph| glyph.0)
    }

    /// Text als Glyphenfolge. Steuerzeichen und fehlende Leerzeichen werden
    /// zum Leerzeichen; fehlende Zeichen erst zu [`FALLBACKS`], dann zu `?`.
    pub fn glyphs(&self, text: &str) -> Vec<Glyph> {
        let mut out = Vec::with_capacity(text.len());
        for c in text.chars() {
            self.push_glyph(c, &mut out);
        }
        out
    }

    fn push_glyph(&self, c: char, out: &mut Vec<Glyph>) {
        let c = if c.is_control() { ' ' } else { c };
        if let Some(id) = self.glyph_id(c) {
            out.push(Glyph { id, text: c });
        } else if c.is_whitespace() {
            self.push_glyph(' ', out);
        } else if let Some((_, replacement)) = FALLBACKS.iter().find(|(from, _)| *from == c) {
            for r in replacement.chars() {
                self.push_glyph(r, out);
            }
        } else if let Some(id) = self.glyph_id('?') {
            out.push(Glyph { id, text: '?' });
        }
    }

    /// Vorschub einer Glyphe in 1/1000 em, wie PDF-Breiten ihn erwarten.
    pub fn advance(&self, id: u16) -> u32 {
        let advance = self.face.glyph_hor_advance(GlyphId(id)).unwrap_or(0);
        self.to_thousandths(i32::from(advance)).unsigned_abs()
    }

    /// Breite eines Texts in Punkten bei `size`.
    pub fn width(&self, text: &str, size: f32) -> f32 {
        let units: u32 = self
            .glyphs(text)
            .iter()
            .map(|glyph| self.advance(glyph.id))
            .sum();
        #[allow(clippy::cast_precision_loss)]
        let em = units as f32 / 1000.0;
        em * size
    }

    /// Schrift-Einheiten auf 1/1000 em.
    pub fn to_thousandths(&self, units: i32) -> i32 {
        let per_em = i32::from(self.face.units_per_em().max(1));
        (units * 1000 + per_em / 2).div_euclid(per_em)
    }

    /// Oberlänge in 1/1000 em.
    pub fn ascent(&self) -> i32 {
        self.to_thousandths(i32::from(self.face.ascender()))
    }

    /// Unterlänge in 1/1000 em (negativ).
    pub fn descent(&self) -> i32 {
        self.to_thousandths(i32::from(self.face.descender()))
    }

    fn cap_units(&self) -> i16 {
        self.face
            .capital_height()
            .unwrap_or_else(|| self.face.ascender())
    }

    /// Versalhöhe in 1/1000 em.
    pub fn cap_height(&self) -> i32 {
        self.to_thousandths(i32::from(self.cap_units()))
    }

    /// Versalhöhe in Punkten bei `size`.
    pub fn cap_height_at(&self, size: f32) -> f32 {
        f32::from(self.cap_units()) / f32::from(self.face.units_per_em().max(1)) * size
    }

    /// Umriss aller Glyphen in 1/1000 em: `[x_min, y_min, x_max, y_max]`.
    pub fn bounding_box(&self) -> [i32; 4] {
        let rect = self.face.global_bounding_box();
        [rect.x_min, rect.y_min, rect.x_max, rect.y_max]
            .map(|value| self.to_thousandths(i32::from(value)))
    }

    /// Kursivwinkel in Grad.
    pub fn italic_angle(&self) -> f32 {
        self.face.italic_angle()
    }

    /// Schriftdatei mit nur den Umrissen von `used` (plus `.notdef` und
    /// allen Bestandteilen zusammengesetzter Glyphen). Lässt sich die Datei
    /// nicht zerlegen, kommt sie unverändert zurück.
    pub fn subset(&self, used: &BTreeSet<u16>) -> Vec<u8> {
        subset(self.data, used).unwrap_or_else(|| self.data.to_vec())
    }
}

// --- TrueType-Teilmenge --------------------------------------------------------

/// Tabellen, die ein PDF-Betrachter für eine eingebettete `CIDFontType2`
/// braucht. Alles andere (Layout-Tabellen, Signaturen) fällt weg.
const KEPT_TABLES: [&[u8; 4]; 14] = [
    b"OS/2", b"cmap", b"cvt ", b"fpgm", b"gasp", b"glyf", b"head", b"hhea", b"hmtx", b"loca",
    b"maxp", b"name", b"post", b"prep",
];

fn read_u16(data: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_be_bytes(data.get(at..at + 2)?.try_into().ok()?))
}

fn read_u32(data: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_be_bytes(data.get(at..at + 4)?.try_into().ok()?))
}

/// Tabellenverzeichnis: Tag und Inhalt je Tabelle.
fn tables(data: &[u8]) -> Option<Vec<([u8; 4], &[u8])>> {
    let count = usize::from(read_u16(data, 4)?);
    (0..count)
        .map(|index| {
            let record = 12 + index * 16;
            let tag: [u8; 4] = data.get(record..record + 4)?.try_into().ok()?;
            let offset = usize::try_from(read_u32(data, record + 8)?).ok()?;
            let length = usize::try_from(read_u32(data, record + 12)?).ok()?;
            Some((tag, data.get(offset..offset + length)?))
        })
        .collect()
}

fn table<'a>(tables: &[([u8; 4], &'a [u8])], tag: [u8; 4]) -> Option<&'a [u8]> {
    tables
        .iter()
        .find(|(name, _)| *name == tag)
        .map(|(_, body)| *body)
}

/// Start und Ende jeder Glyphe in `glyf`, aus `loca`.
fn glyph_ranges(loca: &[u8], long: bool, glyph_count: usize) -> Option<Vec<(usize, usize)>> {
    let offset = |index: usize| -> Option<usize> {
        if long {
            usize::try_from(read_u32(loca, index * 4)?).ok()
        } else {
            Some(usize::from(read_u16(loca, index * 2)?) * 2)
        }
    };
    (0..glyph_count)
        .map(|index| Some((offset(index)?, offset(index + 1)?)))
        .collect()
}

/// Glyph-IDs, aus denen eine zusammengesetzte Glyphe besteht.
fn components(glyph: &[u8]) -> Vec<u16> {
    const ARGS_ARE_WORDS: u16 = 0x0001;
    const HAS_SCALE: u16 = 0x0008;
    const MORE_COMPONENTS: u16 = 0x0020;
    const HAS_XY_SCALE: u16 = 0x0040;
    const HAS_TWO_BY_TWO: u16 = 0x0080;

    let mut out = Vec::new();
    let composite = read_u16(glyph, 0).is_some_and(|contours| contours & 0x8000 != 0);
    if !composite {
        return out;
    }
    let mut at = 10;
    while let (Some(flags), Some(id)) = (read_u16(glyph, at), read_u16(glyph, at + 2)) {
        out.push(id);
        at += 4 + if flags & ARGS_ARE_WORDS != 0 { 4 } else { 2 };
        at += if flags & HAS_SCALE != 0 {
            2
        } else if flags & HAS_XY_SCALE != 0 {
            4
        } else if flags & HAS_TWO_BY_TWO != 0 {
            8
        } else {
            0
        };
        if flags & MORE_COMPONENTS == 0 {
            break;
        }
    }
    out
}

fn checksum(bytes: &[u8]) -> u32 {
    bytes.chunks(4).fold(0u32, |sum, chunk| {
        let mut word = [0u8; 4];
        word[..chunk.len()].copy_from_slice(chunk);
        sum.wrapping_add(u32::from_be_bytes(word))
    })
}

fn subset(data: &[u8], used: &BTreeSet<u16>) -> Option<Vec<u8>> {
    let tables = tables(data)?;
    let head = table(&tables, *b"head")?;
    let glyph_count = usize::from(read_u16(table(&tables, *b"maxp")?, 4)?);
    let long = read_u16(head, 50)? == 1;
    let glyf = table(&tables, *b"glyf")?;
    let ranges = glyph_ranges(table(&tables, *b"loca")?, long, glyph_count)?;

    let mut keep: BTreeSet<u16> = BTreeSet::new();
    let mut pending: Vec<u16> = std::iter::once(0).chain(used.iter().copied()).collect();
    while let Some(id) = pending.pop() {
        let Some(&(start, end)) = ranges.get(usize::from(id)) else {
            continue;
        };
        if keep.insert(id) {
            pending.extend(components(glyf.get(start..end)?));
        }
    }

    let mut new_glyf = Vec::new();
    let mut new_loca = Vec::with_capacity((glyph_count + 1) * 4);
    for (index, &(start, end)) in ranges.iter().enumerate() {
        new_loca.extend_from_slice(&u32::try_from(new_glyf.len()).ok()?.to_be_bytes());
        if u16::try_from(index).is_ok_and(|id| keep.contains(&id)) {
            new_glyf.extend_from_slice(glyf.get(start..end)?);
            while new_glyf.len() % 4 != 0 {
                new_glyf.push(0);
            }
        }
    }
    new_loca.extend_from_slice(&u32::try_from(new_glyf.len()).ok()?.to_be_bytes());

    let mut new_head = head.to_vec();
    new_head.get_mut(8..12)?.copy_from_slice(&[0; 4]);
    new_head
        .get_mut(50..52)?
        .copy_from_slice(&1u16.to_be_bytes());
    // Format 3 trägt keine Glyphnamen; die braucht ein PDF nicht.
    let mut new_post = table(&tables, *b"post")?.get(..32)?.to_vec();
    new_post[..4].copy_from_slice(&0x0003_0000u32.to_be_bytes());

    let mut out_tables: Vec<([u8; 4], Vec<u8>)> = tables
        .iter()
        .filter(|(tag, _)| KEPT_TABLES.contains(&tag))
        .map(|(tag, body)| {
            let body = match tag {
                b"glyf" => new_glyf.clone(),
                b"loca" => new_loca.clone(),
                b"head" => new_head.clone(),
                b"post" => new_post.clone(),
                _ => body.to_vec(),
            };
            (*tag, body)
        })
        .collect();
    out_tables.sort_by_key(|(tag, _)| *tag);
    Some(write_font(&out_tables))
}

fn write_font(tables: &[([u8; 4], Vec<u8>)]) -> Vec<u8> {
    let count = u16::try_from(tables.len()).unwrap_or(u16::MAX);
    let entry_selector = count.max(1).ilog2();
    let search_range = (1u16 << entry_selector) * 16;
    let mut out = Vec::new();
    out.extend_from_slice(&0x0001_0000u32.to_be_bytes());
    for value in [
        count,
        search_range,
        u16::try_from(entry_selector).unwrap_or(0),
        count * 16 - search_range,
    ] {
        out.extend_from_slice(&value.to_be_bytes());
    }

    let mut offset = 12 + tables.len() * 16;
    let mut head_offset = None;
    for (tag, body) in tables {
        if tag == b"head" {
            head_offset = Some(offset);
        }
        out.extend_from_slice(tag);
        out.extend_from_slice(&checksum(body).to_be_bytes());
        out.extend_from_slice(&u32::try_from(offset).unwrap_or(0).to_be_bytes());
        out.extend_from_slice(&u32::try_from(body.len()).unwrap_or(0).to_be_bytes());
        offset += body.len().next_multiple_of(4);
    }
    for (_, body) in tables {
        out.extend_from_slice(body);
        out.resize(out.len().next_multiple_of(4), 0);
    }
    if let Some(head) = head_offset {
        let adjustment = 0xB1B0_AFBAu32.wrapping_sub(checksum(&out));
        out[head + 8..head + 12].copy_from_slice(&adjustment.to_be_bytes());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brand;
    use steps_i18n::{Key, Locale};

    fn faces() -> [&'static Font; 2] {
        [Font::regular(), Font::bold()]
    }

    #[test]
    fn bundled_fonts_are_the_brand_font() {
        for font in faces() {
            assert_eq!(name(&font.face, name_id::FAMILY), brand::font().family);
            assert!(font
                .postscript_name()
                .starts_with("AtkinsonHyperlegibleNext-"));
        }
    }

    #[test]
    fn covers_every_bundled_locale_and_more() {
        let mut texts: Vec<String> = Locale::all()
            .flat_map(|locale| Key::ALL.iter().map(move |key| locale.text(*key).to_owned()))
            .collect();
        texts.push("ÄÖÜäöüß „“ ‚‘ «» ¿¡ ñ ç é è ê ë à â î ï ô û ù ÿ œ æ ã õ €".to_owned());
        texts.push("Zażółć gęślą jaźń ŁŚŹŻ ő ű č ř ž".to_owned());
        for font in faces() {
            for text in &texts {
                let fallback = font.glyph_id('?');
                for c in text.chars().filter(|c| !matches!(c, ' ' | '\n' | '\t')) {
                    let id = font.glyph_id(c);
                    assert!(
                        id.is_some() && (c == '?' || id != fallback),
                        "{} lacks {c:?} in {text}",
                        font.postscript_name()
                    );
                }
            }
        }
    }

    #[test]
    fn missing_characters_fall_back_instead_of_vanishing() {
        let font = Font::regular();
        let glyphs = font.glyphs("a\tb 日");
        let text: String = glyphs.iter().map(|glyph| glyph.text).collect();
        assert_eq!(text, "a b ?");
        let arrow: String = font.glyphs("a→b").iter().map(|glyph| glyph.text).collect();
        assert!(arrow == "a→b" || arrow == "a->b", "{arrow}");
    }

    #[test]
    fn widths_follow_the_font_metrics() {
        let font = Font::regular();
        assert!(font.width("WWW", 12.0) > font.width("iii", 12.0));
        assert!((font.width("", 12.0)).abs() < f32::EPSILON);
        assert!(Font::bold().width("Speichern", 12.0) > 0.0);
        assert!(font.ascent() > 0 && font.descent() < 0 && font.cap_height() > 0);
    }

    #[test]
    fn subset_is_smaller_and_still_parses_with_the_same_glyphs() {
        let font = Font::regular();
        let used: BTreeSet<u16> = font
            .glyphs("Klicke auf „Ä“ → Łódź")
            .iter()
            .map(|g| g.id)
            .collect();
        let small = font.subset(&used);
        assert!(small.len() * 3 < font.data.len(), "{} bytes", small.len());
        let parsed = ttf_parser::Face::parse(&small, 0).expect("subset parses");
        assert_eq!(parsed.number_of_glyphs(), font.face.number_of_glyphs());
        for id in &used {
            assert_eq!(
                parsed.glyph_hor_advance(GlyphId(*id)),
                font.face.glyph_hor_advance(GlyphId(*id))
            );
            assert_eq!(
                parsed.glyph_bounding_box(GlyphId(*id)),
                font.face.glyph_bounding_box(GlyphId(*id)),
                "outline of glyph {id} kept"
            );
        }
        let unused = font.glyph_id('Q').expect("Q");
        assert!(!used.contains(&unused));
        assert_eq!(parsed.glyph_bounding_box(GlyphId(unused)), None);
        assert_eq!(checksum(&small), 0xB1B0_AFBA, "whole-font checksum");
    }
}
