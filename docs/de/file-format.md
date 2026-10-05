[English](../en/file-format.md) | Deutsch

# Das `.steps`-Dateiformat

Eine `.steps`-Datei ist ein ZIP-Archiv mit genau einer aufgenommenen Anleitung. Das Format ist offen, damit andere Werkzeuge Anleitungen ohne Hows lesen, umwandeln oder erzeugen können. Die Referenzimplementierung ist das Crate `steps-store` (`core/store/src/lib.rs`, `model.rs` und für die Grenzen `limits.rs`).

Diese Seite beschreibt Schema-Version <!-- fact:store.schema_version -->**1**<!-- /fact -->.

## Aufbau des Archivs

```
guide.steps
├── guide.json              Metadaten und Schritte (UTF-8-JSON, mit Deflate komprimiert)
└── images/
    ├── step-0001.png       Screenshot von Schritt „step-0001“ (unverändert abgelegt)
    ├── step-0002.png
    └── …
```

- `guide.json` ist Pflicht. Eine Datei ohne sie ist ungültig.
- Der Screenshot eines Schritts liegt unter `images/{step.id}.png`. Schritte ohne Screenshot haben keine Datei.
- Die PNGs sind die ursprünglichen, unmarkierten Screenshots. Markierungen stehen als Daten in `guide.json` und werden erst beim Export in Kopien eingezeichnet.
- Lesende Programme ignorieren andere Einträge, Bilder ohne passenden Schritt, Bilder, deren Name keine gültige Schritt-`id` ist, und Einträge in Unterordnern von `images/`.

## `guide.json`

```json
{
  "schema_version": 1,
  "title": "Map a network drive",
  "description": "Two-minute guide for new colleagues.",
  "created_at_ms": 1790742411542,
  "language": "en",
  "environment": { "os": "Windows", "os_version": "10.0.26100" },
  "steps": [
    {
      "id": "step-0001",
      "action": { "type": "click" },
      "timestamp_ms": 1790742412542,
      "position": { "x": 812, "y": 440 },
      "monitor": { "index": 0, "x": 0, "y": 0, "width": 2560, "height": 1440, "scale_factor": 1.5 },
      "element": {
        "name": "This PC",
        "control_type": "TreeItem",
        "window_title": "File Explorer",
        "app_name": "Explorer"
      },
      "text": { "generated": "Click “This PC” in “Explorer”" },
      "overlays": [
        { "type": "rect", "id": "o1", "color": "#E11D48", "stroke": 2.5, "x": 0.1, "y": 0.2, "w": 0.3, "h": 0.1 }
      ]
    },
    {
      "id": "step-0002",
      "action": { "type": "key_combo", "combo": "Ctrl+S" },
      "timestamp_ms": 1790742415210,
      "text": { "generated": "Press Ctrl+S", "override": "Save the file with Ctrl+S" }
    }
  ]
}
```

### Anleitung

| Feld | Typ | Pflicht | Bedeutung |
|---|---|---|---|
| `schema_version` | Ganzzahl | ja | <!-- fact:store.schema_version -->`1`<!-- /fact -->. Lesende Programme lehnen `0`, einen fehlenden Wert oder eine neuere Version als die unterstützte ab. |
| `title` | Zeichenkette | ja | Titel der Anleitung, vom Nutzer änderbar. |
| `description` | Zeichenkette | nein | Freier Text unter dem Titel. |
| `created_at_ms` | Ganzzahl | ja | Erstellungszeit in Millisekunden seit der Unix-Epoche, UTC. |
| `language` | Zeichenkette | ja | Sprache der erzeugten Schritttexte, BCP-47 (`de`, `en`). |
| `environment` | Objekt | nein | `os` und `os_version`, beide optionale Zeichenketten. Nur in Exporten im Fehlerberichtsmodus zu sehen. Die App schreibt `os` und lässt `os_version` vorerst weg. |
| `steps` | Array | nein | Schritte in Anzeigereihenfolge. Fehlt es, gibt es keine Schritte. |

### Schritt

| Feld | Typ | Pflicht | Bedeutung |
|---|---|---|---|
| `id` | Zeichenkette | ja | Eindeutig innerhalb der Anleitung, zugleich der Dateiname des Screenshots. Hows schreibt `step-0001`, `step-0002` und so weiter. Eine ID hat 1 bis <!-- fact:store.limits.max_step_id_chars -->64<!-- /fact --> Zeichen, nur ASCII-Buchstaben, Ziffern, `-` und `_`, und ist kein Windows-Gerätename wie `CON` oder `NUL`. Lesende Programme lehnen eine Datei mit einer anderen ID, einer doppelten ID oder zwei IDs ab, die sich nur in der ASCII-Großschreibung unterscheiden, etwa `step-A` und `step-a`. |
| `action` | Objekt | ja | Was passiert ist, siehe unten. |
| `timestamp_ms` | Ganzzahl | ja | Zeitpunkt der Aktion in Millisekunden seit der Unix-Epoche, UTC. |
| `position` | Objekt | nein | `x`, `y`: ganzzahlige physische Pixel auf dem virtuellen Desktop; negativ auf Bildschirmen links oder oberhalb des Hauptbildschirms. |
| `monitor` | Objekt | nein | Bildschirm bei der Aufnahme: `index`, `x`, `y` (Ursprung auf dem virtuellen Desktop, Standard `0`), `width`, `height` (physische Pixel), `scale_factor` (zum Beispiel `1.5` für 150 %). |
| `element` | Objekt | nein | Das getroffene Element der Oberfläche, siehe unten. |
| `text` | Objekt | ja | `generated`: Text aus der Vorlage; `override`: optionale Änderung durch den Nutzer. Angezeigt wird `override`, falls vorhanden, sonst `generated`. |
| `overlays` | Array | nein | Markierungen, siehe unten. Fehlt es, gibt es keine. |
| `crop` | Objekt | nein | Sichtbares Fenster des Screenshots: `x`, `y`, `w`, `h`, jeweils von 0 bis 1. Fehlt es, gilt das ganze Bild. Die PNG in `images/` bleibt vollständig. Exporte zeigen das Fenster. |

### Aktion

`action.type` wählt die Variante:

| `type` | Weitere Felder | Bedeutung |
|---|---|---|
| `click` | | Linksklick. |
| `double_click` | | Doppelklick (Doppelklick-Zeit und -Abstand des Systems). |
| `right_click` | | Rechtsklick. |
| `key_combo` | `combo` (Zeichenkette) | Tastenkürzel mit Modifikatortaste, mit `+` verbunden, zum Beispiel `Ctrl+Shift+S`. Normales Tippen wird nie so gespeichert. |
| `scroll` | `direction` (`up` \| `down` \| `left` \| `right`), `amount` (Ganzzahl) | Scrollen in einem Fenster und einer Richtung, zusammengefasst bis zu einer Pause. `amount` zählt Rasterstufen des Mausrads; Touchpad-Bewegung wird summiert und auf ganze Stufen aufgerundet. |
| `text_input` | | In `element` wurde getippt. Die getippten Zeichen werden nie aufgenommen. |
| `manual_note` | | Von Hand oder mit `steps-cli from-script` hinzugefügter Schritt. |

Die Werte in `combo` stehen immer in der sprachneutralen Form (`Ctrl`, `Shift`), auch wenn die Oberfläche `Strg` und `Umschalt` zeigt.

### Element

Alle Felder sind optionale Zeichenketten.

| Feld | Quelle | Bedeutung |
|---|---|---|
| `name` | UI Automation | Sichtbarer Name, zum Beispiel die Beschriftung einer Schaltfläche. |
| `control_type` | UI Automation | `Button`, `MenuItem`, `TreeItem` und so weiter. |
| `automation_id` | UI Automation | Vom Entwickler vergebene Kennung des Elements. |
| `window_title` | Fenster | Titel des Hauptfensters. Kann Kunden- oder Dokumentnamen enthalten. |
| `app_name` | Prozess | Name der Anwendung. |
| `selector`, `aria_label`, `url`, `page_title` | Browser-Erweiterung | Für die geplante Erweiterung reserviert; wird heute nicht geschrieben. |

### Markierungen

Koordinaten sind Anteile an Breite und Höhe des Bilds (0 bis 1) und überstehen so jede Skalierung. `color` ist `#RRGGBB`; `stroke` und `size` sind Bildpixel bei 100 % Zoom. Jede Markierung hat eine feste `id`.

| `type` | Felder | Dargestellt als |
|---|---|---|
| `rect` | `color`, `stroke`, `x`, `y`, `w`, `h` | Umriss eines Rechtecks. |
| `arrow` | `color`, `stroke`, `x1`, `y1`, `x2`, `y2` | Linie mit gefüllter Spitze bei `x2`/`y2`. |
| `pen` | `color`, `stroke`, `points` (Array aus `[x, y]`) | Freihandlinie. |
| `highlight` | `color`, `opacity` (0 bis 1), `x`, `y`, `w`, `h` | Durchscheinend gefülltes Rechteck. |
| `text` | `color`, `size`, `x`, `y`, `text` | Einzeilige Beschriftung, oben links verankert. |
| `circle` | `color`, `stroke`, `cx`, `cy`, `r` | Umriss eines Kreises; `r` ist ein Anteil an der **Breite** des Bilds. |
| `blur` | `x`, `y`, `w`, `h` | Bereich, der beim Export unlesbar gemacht (verpixelt) wird. |

Standards von Hows: Farbe neuer Markierungen <!-- fact:marks.color -->`#E11D48`<!-- /fact -->, Textmarker <!-- fact:marks.highlight_color -->`#FDE047`<!-- /fact --> mit Deckkraft <!-- fact:marks.highlight_opacity -->`0.4`<!-- /fact -->, Linienstärke für Formen <!-- fact:marks.stroke -->`2.5`<!-- /fact -->, für den Stift <!-- fact:marks.pen_stroke -->`3`<!-- /fact -->, Textgröße <!-- fact:marks.text_size -->`14`<!-- /fact -->.

## Grenzen

Lesende Programme prüfen jede Datei, bevor sie sie verwenden. Eine Datei, die eine dieser Grenzen überschreitet, gilt als beschädigt und wird als Ganzes abgelehnt; Hows öffnet nie einen Teil davon. Die Grenzen lassen Anleitungen, die Hows aufnimmt, viel Raum. Größen zählen die tatsächlich entpackten Bytes, nicht die Größen, die das ZIP angibt. Hows prüft dieselben Grenzen vor dem Speichern und schreibt deshalb nie eine Datei, die es ablehnen würde.

| Was | Grenze |
|---|---|
| Einträge im ZIP | <!-- fact:store.limits.max_zip_entries -->10000<!-- /fact --> |
| `guide.json`, entpackt | <!-- fact:store.limits.max_guide_json_bytes mib -->16<!-- /fact --> MiB |
| Ein Screenshot | <!-- fact:store.limits.max_image_bytes mib -->32<!-- /fact --> MiB |
| Alle Screenshots einer Datei zusammen | <!-- fact:store.limits.max_total_image_bytes mib -->1024<!-- /fact --> MiB |
| Breite oder Höhe eines Screenshots laut PNG-Kopf | <!-- fact:store.limits.max_image_side -->16384<!-- /fact --> Pixel |
| Pixel eines Screenshots laut PNG-Kopf | <!-- fact:store.limits.max_image_pixels -->50000000<!-- /fact --> |
| Schritte | <!-- fact:store.limits.max_steps -->5000<!-- /fact --> |
| Zeichen einer Schritt-`id` | <!-- fact:store.limits.max_step_id_chars -->64<!-- /fact --> |
| Markierungen pro Schritt | <!-- fact:store.limits.max_overlays_per_step -->500<!-- /fact --> |
| Punkte einer `pen`-Markierung | <!-- fact:store.limits.max_pen_points -->10000<!-- /fact --> |
| Zeichen von `description`, Schritttexten, dem `name` des Elements und dem `text` einer `text`-Markierung | <!-- fact:store.limits.max_text_chars -->10000<!-- /fact --> |
| Zeichen jeder anderen Zeichenkette, etwa `title`, `window_title`, `app_name` oder die `color` einer Markierung | <!-- fact:store.limits.max_label_chars -->2000<!-- /fact --> |
| Koordinaten und Größen von Markierungen (`x`, `y`, `w`, `h`, `x1` bis `y2`, `cx`, `cy`, `r`, `points`) | von <!-- fact:store.limits.min_overlay_coord -->-2<!-- /fact --> bis <!-- fact:store.limits.max_overlay_coord -->3<!-- /fact -->, eine Markierung darf also zwei Bildbreiten oder -höhen über den Rand reichen |
| `stroke` | 0 bis <!-- fact:store.limits.max_overlay_stroke -->32<!-- /fact --> Pixel |
| `size` einer `text`-Markierung | 0 bis <!-- fact:store.limits.max_overlay_text_size -->64<!-- /fact --> Pixel |
| `opacity` einer `highlight`-Markierung | 0 bis 1 |

Die Grenzen beschränken die Arbeit des eigenen Codes von Hows. Exporte dekodieren Screenshots und zeichnen Markierungen hinein nur innerhalb dieser Grenzen. Der Editor in der App gibt jeden Screenshot unverändert an die Web-Ansicht von Windows (WebView2) weiter, die ihn selbst dekodiert. Für die Web-Ansicht gilt nur die Byte-Grenze pro Screenshot; die Pixelgrenzen stammen aus dem PNG-Kopf, ein Screenshot, der kein PNG ist, kommt also ungeprüft durch.

## Regeln für Kompatibilität

- Unbekannte JSON-Felder werden ignoriert. Dateien neuerer Hows-Versionen mit derselben Schema-Version bleiben so lesbar.
- Markierungen mit unbekanntem `type` werden einzeln übersprungen; der Rest des Schritts wird geladen.
- Optionale Felder dürfen fehlen; schreibende Programme lassen leere optionale Werte weg.
- Eine inkompatible Änderung erhöht `schema_version`. Lesende Programme müssen unbekannte Versionen ablehnen, statt zu raten.

## Kleinste gültige Datei

Ein ZIP, das nur diese `guide.json` enthält, ist eine gültige Anleitung mit einem Textschritt ohne Screenshot:

```json
{
  "schema_version": 1,
  "title": "Hello",
  "created_at_ms": 0,
  "language": "en",
  "steps": [
    { "id": "step-0001", "action": { "type": "manual_note" }, "timestamp_ms": 0, "text": { "generated": "Say hello." } }
  ]
}
```

---

[Zur Übersicht](README.md) · Weiter: [Probleme lösen](troubleshooting.md)
