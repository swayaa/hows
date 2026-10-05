//! Datenmodell für `guide.json` (Schema-Version 1), siehe Spec §4.

use serde::{Deserialize, Serialize};

/// Aktuelle Schema-Version des `guide.json`-Formats.
pub const SCHEMA_VERSION: u32 = 1;

/// Wurzelobjekt von `guide.json`: Metadaten und Schrittliste einer Aufnahme.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Guide {
    /// Version des Dateischemas (aktuell `1`).
    pub schema_version: u32,
    /// Titel des Guides (vom User editierbar).
    pub title: String,
    /// Optionale Beschreibung des Guides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Erstellt-Zeitpunkt als Unix-Epoche in Millisekunden (UTC).
    pub created_at_ms: u64,
    /// Sprache der generierten Schritt-Texte (BCP-47, z. B. `de`, `en`).
    pub language: String,
    /// Optionale Umgebungsinfo für den Bug-Report-Modus.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<Environment>,
    /// Aufgenommene Schritte in Anzeige-Reihenfolge.
    #[serde(default)]
    pub steps: Vec<Step>,
}

impl Guide {
    /// Erzeugt einen leeren Guide mit aktueller Schema-Version.
    pub fn new(title: impl Into<String>, created_at_ms: u64, language: impl Into<String>) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            title: title.into(),
            description: None,
            created_at_ms,
            language: language.into(),
            environment: None,
            steps: Vec::new(),
        }
    }
}

/// Umgebungsinfo für den Bug-Report-Modus; jedes Feld ist optional.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Environment {
    /// Betriebssystem (z. B. `Windows`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
    /// OS-Versionsstring (z. B. `10.0.26100`); die App füllt ihn derzeit nicht.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os_version: Option<String>,
}

/// Ein einzelner aufgenommener Schritt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Step {
    /// Eindeutige Schritt-ID; zugleich Dateiname des Screenshots
    /// (`images/{id}.png`).
    pub id: String,
    /// Aufgenommene Aktion inklusive aktionsspezifischer Daten.
    pub action: Action,
    /// Zeitpunkt der Aktion als Unix-Epoche in Millisekunden (UTC).
    pub timestamp_ms: u64,
    /// Klickposition in physischen Pixeln (virtueller Desktop; kann bei
    /// Multi-Monitor-Setups negativ sein). `None` z. B. bei `manual_note`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,
    /// Monitor, auf dem die Aktion stattfand.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub monitor: Option<Monitor>,
    /// Metadaten des Ziel-Elements (UIA; per Extension ersetzbar).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element: Option<ElementInfo>,
    /// Schritt-Text: generiert + optionaler User-Override.
    pub text: StepText,
    /// Greenshot-lite Markierungen (Review); Koordinaten normalisiert 0–1.
    /// Fehlendes Feld ≡ leere Liste (alte `.steps`-Dateien bleiben lesbar).
    /// Unbekannte `type`-Werte werden beim Lesen übersprungen (forward-safe).
    #[serde(
        default,
        skip_serializing_if = "Vec::is_empty",
        deserialize_with = "deserialize_overlays"
    )]
    pub overlays: Vec<Overlay>,
    /// Sichtbarer Ausschnitt des Screenshots, normalisiert 0–1.
    /// `None` ist das ganze Bild. Die PNG-Bytes bleiben vollständig.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crop: Option<Crop>,
}

/// Fenster auf dem Original-Screenshot. Koordinaten sind 0–1, wie bei Markierungen.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Crop {
    /// Linke Kante.
    pub x: f64,
    /// Obere Kante.
    pub y: f64,
    /// Breite.
    pub w: f64,
    /// Höhe.
    pub h: f64,
}

impl Crop {
    /// Klemmt das Fenster ins Bild. Das ganze Bild wird `None`.
    /// Nicht-endliche Zahlen sind ein Fehler.
    pub fn normalized(self) -> Result<Option<Self>, ()> {
        if ![self.x, self.y, self.w, self.h]
            .into_iter()
            .all(f64::is_finite)
        {
            return Err(());
        }
        let (mut x, mut w) = non_negative_span(self.x, self.w);
        let (mut y, mut h) = non_negative_span(self.y, self.h);
        const MIN: f64 = 0.02;
        w = w.clamp(MIN, 1.0);
        h = h.clamp(MIN, 1.0);
        x = x.clamp(0.0, 1.0 - w);
        y = y.clamp(0.0, 1.0 - h);
        if x <= 1e-3 && y <= 1e-3 && w >= 1.0 - 1e-3 && h >= 1.0 - 1e-3 {
            return Ok(None);
        }
        Ok(Some(Self { x, y, w, h }))
    }
}

fn non_negative_span(origin: f64, span: f64) -> (f64, f64) {
    if span < 0.0 {
        (origin + span, -span)
    } else {
        (origin, span)
    }
}

fn deserialize_overlays<'de, D>(deserializer: D) -> Result<Vec<Overlay>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let values = Vec::<serde_json::Value>::deserialize(deserializer)?;
    Ok(values
        .into_iter()
        .filter_map(|value| serde_json::from_value::<Overlay>(value).ok())
        .collect())
}

/// Nicht-destruktive Markierung auf einem Schritt-Screenshot.
///
/// Koordinaten sind relativ zur Bildbreite/-höhe (0–1), damit Zoom und DPI
/// die Geometrie nicht verzerren. Eingebrannt wird erst beim Raster-Export.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Overlay {
    /// Achsenparalleles Rechteck ohne Füllung, von Ecke zu Ecke.
    Rect {
        /// Stabile Overlay-ID (Undo/Clear).
        id: String,
        /// Strichfarbe (`#RRGGBB`).
        color: String,
        /// Strichstärke bei 1× Bildpixeln.
        stroke: f64,
        /// Linke Kante (0–1).
        x: f64,
        /// Obere Kante (0–1).
        y: f64,
        /// Breite (0–1).
        w: f64,
        /// Höhe (0–1).
        h: f64,
    },
    /// Gerade Linie mit gefüllter Pfeilspitze am Ziel.
    Arrow {
        /// Stabile Overlay-ID (Undo/Clear).
        id: String,
        /// Strichfarbe (`#RRGGBB`).
        color: String,
        /// Strichstärke bei 1× Bildpixeln.
        stroke: f64,
        /// Start-X (0–1).
        x1: f64,
        /// Start-Y (0–1).
        y1: f64,
        /// Ziel-X / Spitze (0–1).
        x2: f64,
        /// Ziel-Y / Spitze (0–1).
        y2: f64,
    },
    /// Freihand-Polyline.
    Pen {
        /// Stabile Overlay-ID (Undo/Clear).
        id: String,
        /// Strichfarbe (`#RRGGBB`).
        color: String,
        /// Strichstärke bei 1× Bildpixeln.
        stroke: f64,
        /// Stützpunkte als `[x, y]` in 0–1.
        points: Vec<[f64; 2]>,
    },
    /// Transluzenter Marker (gefülltes Rechteck, kein Strich).
    Highlight {
        /// Stabile Overlay-ID (Undo/Clear).
        id: String,
        /// Füllfarbe (`#RRGGBB`).
        color: String,
        /// Deckkraft 0–1 (Standard aus `marks.json`).
        opacity: f64,
        /// Linke Kante (0–1).
        x: f64,
        /// Obere Kante (0–1).
        y: f64,
        /// Breite (0–1).
        w: f64,
        /// Höhe (0–1).
        h: f64,
    },
    /// Einzeiliges Label (Klick-platzieren).
    Text {
        /// Stabile Overlay-ID (Undo/Clear).
        id: String,
        /// Textfarbe (`#RRGGBB`).
        color: String,
        /// Schriftgröße bei 1× (px).
        size: f64,
        /// Anker-X (0–1), links.
        x: f64,
        /// Anker-Y (0–1), Baseline-Nähe / Oberkante der Zeile.
        y: f64,
        /// Sichtbarer Text (einzeilig).
        text: String,
    },
    /// Roter Callout-Kreis (nur Strich). `r` ist relativ zur Bildbreite.
    Circle {
        /// Stabile Overlay-ID (Undo/Clear).
        id: String,
        /// Strichfarbe (`#RRGGBB`).
        color: String,
        /// Strichstärke bei 1× Bildpixeln.
        stroke: f64,
        /// Mittelpunkt-X (0–1).
        cx: f64,
        /// Mittelpunkt-Y (0–1).
        cy: f64,
        /// Radius relativ zur Bildbreite (0–1).
        r: f64,
    },
    /// Harte Unschärfe-/Pixelate-Maske über einer Region (PII).
    Blur {
        /// Stabile Overlay-ID (Undo/Clear).
        id: String,
        /// Linke Kante (0–1).
        x: f64,
        /// Obere Kante (0–1).
        y: f64,
        /// Breite (0–1).
        w: f64,
        /// Höhe (0–1).
        h: f64,
    },
}

/// Aktionsart eines Schritts. Tastatur erfasst nie Klartext, nur
/// Modifier-Kombinationen (Spec §5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    /// Einfacher Linksklick.
    Click,
    /// Doppelklick.
    DoubleClick,
    /// Rechtsklick.
    RightClick,
    /// Tastenkombination mit Modifier, z. B. `Ctrl+S` (nie reiner Klartext).
    KeyCombo {
        /// Normalisierte Kombination, Bestandteile mit `+` verbunden.
        combo: String,
    },
    /// Zusammengefasstes Scrollen: ein Schritt pro Fenster und Richtung,
    /// bis eine Scroll-Pause eine neue Gruppe beginnt. Der generierte Satz
    /// nennt das Fenster und `amount`.
    Scroll {
        /// Scroll-Richtung.
        direction: ScrollDirection,
        /// Rad-Rasten; Touchpad-Bruchteile summiert und aufgerundet.
        amount: u32,
    },
    /// Zusammengefasste Texteingabe. Ohne Inhalt, nur das Ziel-Element.
    TextInput,
    /// Manuell im Editor eingefügter Schritt.
    ManualNote,
}

/// Scroll-Richtung.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScrollDirection {
    /// Nach oben.
    Up,
    /// Nach unten.
    Down,
    /// Nach links.
    Left,
    /// Nach rechts.
    Right,
}

/// Punkt in physischen Pixeln.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    /// X-Koordinate.
    pub x: i32,
    /// Y-Koordinate.
    pub y: i32,
}

/// Monitor-Metadaten zum Zeitpunkt der Aufnahme (Per-Monitor-DPI, Spec §5).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Monitor {
    /// Monitor-Index im System.
    pub index: u32,
    /// X-Ursprung im virtuellen Desktop (physische Pixel; kann negativ sein).
    /// Nötig, um Klickpositionen in Bild-Pixelkoordinaten umzurechnen.
    #[serde(default)]
    pub x: i32,
    /// Y-Ursprung im virtuellen Desktop (physische Pixel; kann negativ sein).
    #[serde(default)]
    pub y: i32,
    /// Breite in physischen Pixeln.
    pub width: u32,
    /// Höhe in physischen Pixeln.
    pub height: u32,
    /// DPI-Skalierungsfaktor (z. B. `1.5` für 150 %).
    pub scale_factor: f64,
}

/// Metadaten des Ziel-Elements. UIA-Felder werden bei Browser-Aufnahmen
/// durch reichere Extension-Daten ersetzt (Phase 2, Spec §7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ElementInfo {
    /// Element-Name (z. B. Button-Beschriftung).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Kontrolltyp (z. B. `Button`, `MenuItem`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub control_type: Option<String>,
    /// UIA-AutomationId.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub automation_id: Option<String>,
    /// Fenstertitel; kann Kundennamen enthalten.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub window_title: Option<String>,
    /// App-Name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_name: Option<String>,
    /// CSS-Selektor (nur via Browser-Extension).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selector: Option<String>,
    /// ARIA-Label (nur via Browser-Extension).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aria_label: Option<String>,
    /// Seiten-URL (nur via Browser-Extension).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Seitentitel (nur via Browser-Extension).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub page_title: Option<String>,
}

/// Schritt-Text: deterministisch generiert plus optionaler User-Override.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepText {
    /// Deterministisch generierter Template-Text.
    pub generated: String,
    /// User-Edit; hat beim Rendern Vorrang vor `generated`.
    #[serde(rename = "override", default, skip_serializing_if = "Option::is_none")]
    pub override_text: Option<String>,
}

impl StepText {
    /// Erzeugt einen Text ohne Override.
    pub fn generated(text: impl Into<String>) -> Self {
        Self {
            generated: text.into(),
            override_text: None,
        }
    }

    /// Effektiver Anzeigetext: Override, falls vorhanden, sonst generiert.
    pub fn effective(&self) -> &str {
        self.override_text.as_deref().unwrap_or(&self.generated)
    }
}
