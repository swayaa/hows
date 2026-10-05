English | [Deutsch](../de/file-format.md)

# The `.steps` file format

A `.steps` file is a ZIP archive holding one recorded guide. The format is open so other tools can read, convert, or generate guides without Hows. The reference implementation is the `steps-store` crate (`core/store/src/lib.rs`, `core/store/src/model.rs`, and `core/store/src/limits.rs` for the limits).

This document describes schema version <!-- fact:store.schema_version -->**1**<!-- /fact -->.

## Container layout

```
guide.steps
├── guide.json              metadata and steps (UTF-8 JSON, deflate-compressed)
└── images/
    ├── step-0001.png       screenshot of step "step-0001" (stored, not recompressed)
    ├── step-0002.png
    └── …
```

- `guide.json` is required. A file without it is invalid.
- A step's screenshot lives at `images/{step.id}.png`. Steps without a screenshot have no file.
- PNGs are the original, unannotated screenshots. Annotations are stored as data in `guide.json` and burned into copies only when exporting.
- Readers ignore other entries, images without a matching step, images whose name is not a valid step `id`, and entries in subfolders of `images/`.

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

### Guide

| Field | Type | Required | Meaning |
|---|---|---|---|
| `schema_version` | integer | yes | <!-- fact:store.schema_version -->`1`<!-- /fact -->. Readers reject `0`, a missing value, or a version newer than they support. |
| `title` | string | yes | Guide title, editable by the user. |
| `description` | string | no | Free text under the title. |
| `created_at_ms` | integer | yes | Creation time, Unix epoch milliseconds, UTC. |
| `language` | string | yes | Language of the generated step texts, BCP-47 (`de`, `en`). |
| `environment` | object | no | `os` and `os_version`, both optional strings. Shown only in bug-report exports. The app writes `os` and leaves out `os_version` for now. |
| `steps` | array | no | Steps in display order. Missing means empty. |

### Step

| Field | Type | Required | Meaning |
|---|---|---|---|
| `id` | string | yes | Unique within the guide; also the screenshot file name. Hows writes `step-0001`, `step-0002`, …. An id has 1 to <!-- fact:store.limits.max_step_id_chars -->64<!-- /fact --> characters, only ASCII letters, digits, `-`, and `_`, and is not a Windows device name such as `CON` or `NUL`. Readers reject a file with any other id, with an id used twice, or with two ids that differ only by ASCII letter case, such as `step-A` and `step-a`. |
| `action` | object | yes | What happened, see below. |
| `timestamp_ms` | integer | yes | Time of the action, Unix epoch milliseconds, UTC. |
| `position` | object | no | `x`, `y`: integer physical pixels on the virtual desktop; negative on monitors left of or above the primary one. |
| `monitor` | object | no | Monitor at capture time: `index`, `x`, `y` (origin on the virtual desktop, default `0`), `width`, `height` (physical pixels), `scale_factor` (for example, `1.5` for 150%). |
| `element` | object | no | The UI element that was hit, see below. |
| `text` | object | yes | `generated`: template text; `override`: optional user edit. Display `override` if present, else `generated`. |
| `overlays` | array | no | Annotations, see below. Missing means none. |
| `crop` | object | no | Visible window of the screenshot: `x`, `y`, `w`, `h`, each from 0 to 1. Missing means the whole image. The PNG in `images/` stays complete. Exports show the window. |

### Action

`action.type` selects the variant:

| `type` | Extra fields | Meaning |
|---|---|---|
| `click` | | Left click. |
| `double_click` | | Double click (system double-click time and distance). |
| `right_click` | | Right click. |
| `key_combo` | `combo` (string) | Shortcut with a modifier, joined with `+`, for example, `Ctrl+Shift+S`. Plain typing is never stored this way. |
| `scroll` | `direction` (`up` \| `down` \| `left` \| `right`), `amount` (integer) | Scrolling in a single window and scroll direction, merged until a pause. `amount` counts wheel notches; touchpad movement is summed and rounded up to whole notches. |
| `text_input` | | Typing happened in `element`. The typed characters are never recorded. |
| `manual_note` | | Step added by hand or by `steps-cli from-script`. |

`combo` always uses the language-neutral key names (`Ctrl`, `Shift`), whatever language the interface uses.

### Element

All fields are optional strings.

| Field | Source | Meaning |
|---|---|---|
| `name` | UI Automation | Visible name, for example, a button label. |
| `control_type` | UI Automation | `Button`, `MenuItem`, `TreeItem`, … |
| `automation_id` | UI Automation | Developer-assigned element id. |
| `window_title` | Window | Title of the top-level window. May contain customer or document names. |
| `app_name` | Process | Application name. |
| `selector`, `aria_label`, `url`, `page_title` | Browser extension | Reserved for the planned extension; not written today. |

### Overlays

Coordinates are fractions of the image width and height (0 to 1), so they survive scaling. `color` is `#RRGGBB`; `stroke` and `size` are in image pixels at 100% zoom. Every overlay has a stable `id`.

| `type` | Fields | Drawn as |
|---|---|---|
| `rect` | `color`, `stroke`, `x`, `y`, `w`, `h` | Rectangle outline. |
| `arrow` | `color`, `stroke`, `x1`, `y1`, `x2`, `y2` | Line with a filled head at `x2`/`y2`. |
| `pen` | `color`, `stroke`, `points` (array of `[x, y]`) | Freehand polyline. |
| `highlight` | `color`, `opacity` (0 to 1), `x`, `y`, `w`, `h` | Translucent filled rectangle. |
| `text` | `color`, `size`, `x`, `y`, `text` | Single-line label anchored at its top-left corner. |
| `circle` | `color`, `stroke`, `cx`, `cy`, `r` | Circle outline; `r` is a fraction of the image **width**. |
| `blur` | `x`, `y`, `w`, `h` | Region made unreadable (pixelated) on export. |

Hows uses these defaults: mark color <!-- fact:marks.color -->`#E11D48`<!-- /fact -->, highlighter <!-- fact:marks.highlight_color -->`#FDE047`<!-- /fact --> at <!-- fact:marks.highlight_opacity -->`0.4`<!-- /fact --> opacity, shape stroke <!-- fact:marks.stroke -->`2.5`<!-- /fact -->, pen stroke <!-- fact:marks.pen_stroke -->`3`<!-- /fact -->, and text size <!-- fact:marks.text_size -->`14`<!-- /fact -->.

## Limits

Readers check every file before they use it. A file that breaks one of these limits counts as damaged and is rejected as a whole; Hows never opens part of it. The limits leave plenty of room for guides that Hows records. Sizes count the bytes actually unpacked, not the sizes the ZIP declares. Hows checks the same limits before it saves, so it never writes a file it would reject.

| What | Limit |
|---|---|
| Entries in the ZIP | <!-- fact:store.limits.max_zip_entries -->10000<!-- /fact --> |
| `guide.json`, unpacked | <!-- fact:store.limits.max_guide_json_bytes mib -->16<!-- /fact --> MiB |
| One screenshot | <!-- fact:store.limits.max_image_bytes mib -->32<!-- /fact --> MiB |
| All screenshots of a file together | <!-- fact:store.limits.max_total_image_bytes mib -->1024<!-- /fact --> MiB |
| Width or height of a screenshot, from its PNG header | <!-- fact:store.limits.max_image_side -->16384<!-- /fact --> pixels |
| Pixels of a screenshot, from its PNG header | <!-- fact:store.limits.max_image_pixels -->50000000<!-- /fact --> |
| Steps | <!-- fact:store.limits.max_steps -->5000<!-- /fact --> |
| Characters of a step `id` | <!-- fact:store.limits.max_step_id_chars -->64<!-- /fact --> |
| Overlays per step | <!-- fact:store.limits.max_overlays_per_step -->500<!-- /fact --> |
| Points of a `pen` overlay | <!-- fact:store.limits.max_pen_points -->10000<!-- /fact --> |
| Characters of `description`, step texts, the element `name`, and the `text` of a `text` overlay | <!-- fact:store.limits.max_text_chars -->10000<!-- /fact --> |
| Characters of every other string, such as `title`, `window_title`, `app_name`, or an overlay `color` | <!-- fact:store.limits.max_label_chars -->2000<!-- /fact --> |
| Overlay coordinates and sizes (`x`, `y`, `w`, `h`, `x1` to `y2`, `cx`, `cy`, `r`, `points`) | from <!-- fact:store.limits.min_overlay_coord -->-2<!-- /fact --> to <!-- fact:store.limits.max_overlay_coord -->3<!-- /fact -->, so a mark may reach two image widths or heights past the edge |
| `stroke` | 0 to <!-- fact:store.limits.max_overlay_stroke -->32<!-- /fact --> pixels |
| `size` of a `text` overlay | 0 to <!-- fact:store.limits.max_overlay_text_size -->64<!-- /fact --> pixels |
| `opacity` of a `highlight` overlay | 0 to 1 |

The limits bound the work that the code of Hows itself does. Exports decode screenshots and draw overlays into them only within these limits. The editor in the app passes each screenshot unchanged to the Windows web view (WebView2), which decodes it itself. For the web view, only the byte limit per screenshot applies; the pixel limits come from the PNG header, so a screenshot that is not a PNG passes them unchecked.

## Compatibility rules

- Unknown JSON fields are ignored, so files from newer Hows versions with the same schema version stay readable.
- Overlays with an unknown `type` are skipped individually; the rest of the step loads.
- Optional fields may be absent; writers omit empty optional values.
- A breaking change bumps `schema_version`. Readers must refuse versions they do not recognize rather than guessing.

## Minimal valid file

A ZIP with only this `guide.json` is a valid guide with one text step and no screenshot:

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

[Documentation overview](README.md) · Next: [Troubleshooting](troubleshooting.md)
