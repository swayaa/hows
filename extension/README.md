English | [Deutsch](README.de.md)

# Browser extension (planned)

This folder is reserved for a WebExtension (Manifest V3, one codebase for Chrome, Edge, and Firefox) with a native-messaging host.

Without it, browser recordings already work through UI Automation. The extension would add precision: CSS selectors, ARIA labels, page URLs, and page titles. The `.steps` format reserves the fields `selector`, `aria_label`, `url`, and `page_title` for this; see [The `.steps` file format](../docs/en/file-format.md#element).

Nothing is implemented yet. If you want to work on it, open an issue first so we can agree on the messaging protocol.
