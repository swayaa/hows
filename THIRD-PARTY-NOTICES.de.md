[English](THIRD-PARTY-NOTICES.md) | Deutsch

# Hinweise zu Fremdkomponenten

Hows enthält Software und eine Schrift anderer Urheber. Die vollständige Liste mit allen Lizenztexten steht in [THIRD-PARTY-NOTICES.md](THIRD-PARTY-NOTICES.md). Sie bleibt auf Englisch, weil die Lizenztexte nur im englischen Original gelten. Die Installer legen diese Datei und die [MIT-Lizenz](LICENSE) von Hows neben `hows.exe`. Die portable Verteilung ist eine ZIP-Datei mit `hows.exe`, `LICENSE` und der englischen Liste. Das MSI reist zusammen mit dem Archiv `wix-UIExtension-wix3141rtm.zip`, das die Quelle der WiX-Oberfläche und ihre Lizenz enthält.

Die englische Liste nennt auch die Pakete, die npm zum Bauen der Oberfläche installiert, weil `@lucide/svelte` Svelte verlangt. Diese Compiler-Pakete stehen nicht in `hows.exe`.

Die Liste umfasst:

- die Rust-Crates, die in `hows.exe` für 64-Bit-Windows einkompiliert sind, und die Rust-Standardbibliothek;
- die Pakete, die in die Oberfläche der App gebündelt sind;
- die Schrift Atkinson Hyperlegible Next unter der SIL Open Font License 1.1, in der Oberfläche und eingebettet in HTML- und PDF-Exporte;
- die Teile von NSIS und WiX Toolset, die die Installer mitbringen.

Build-Werkzeuge, Entwicklungs- und Testabhängigkeiten sowie Crates, die nur Build-Skripte oder Makros beim Kompilieren nutzen, gehören nicht zur App und stehen nicht in der Liste. Bietet eine Komponente mehrere Lizenzen zur Wahl, nennt die Liste die Lizenz, unter der Hows sie nutzt.

Hows selbst steht unter der [MIT-Lizenz](LICENSE).
