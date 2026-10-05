[English](CHANGELOG.md) | Deutsch

# Änderungsprotokoll

Alle nennenswerten Änderungen an Hows stehen hier. Das Format folgt [Keep a Changelog](https://keepachangelog.com/de/1.1.0/), und das Projekt nutzt [Semantic Versioning](https://semver.org/lang/de/).

## [Unreleased]

## [0.1.0] - 2026-10-06

- Start oder Stopp einer Aufnahme über das Kürzel, während Hows im Hintergrund, minimiert oder im Infobereich ist, lässt ein einziges Fenster und zeigt die Aufnahmeleiste oben. Ein zweiter Start öffnet kein weiteres Hows.
- Ein Klick auf Hows, auch Stopp auf der Aufnahmeleiste, wird nicht aufgenommen.
- Die Bibliothek zeigt nicht mehr „Nur auf diesem PC“ und nicht mehr „Auf diesem PC gespeichert“.
- Die Farbe neuer Markierungen wählst du in der Markierungsleiste; Hows merkt sie sich. Sie steht nicht mehr unter Darstellung.
- Titel, Markieren und Exportieren stehen in einer Zeile. Schritttext und Schritt löschen stehen über dem Screenshot. Enter übernimmt den Schritttext und verlässt das Feld. Umschalt+Enter beginnt eine neue Zeile. Ein Hinweis unter dem Feld sagt das.
- Zuschneiden blendet die Ränder eines Screenshots aus, ohne sie zu löschen. Es sitzt in der Markieren-Leiste. Du ziehst eine Ecke des Rahmens. Ganzes Bild zeigen holt die Ränder zurück. Exporte zeigen das Fenster. Das Bild in der Anleitungsdatei bleibt ganz.
- Ein Scroll-Schritt nennt das Fenster und wie viele Rasterstufen du gescrollt hast, zum Beispiel *Scrolle 3 in „Dokumente“ nach unten*. Scrollst du in diesem Fenster und in dieser Richtung weiter, zählt Hows nur dazu und schaut nicht noch einmal nach, was unter dem Zeiger liegt.
- `Alt+F4` beendet Hows. Jedes andere Schließen der Aufnahmeleiste, zum Beispiel über die Taskleiste, pausiert die Aufnahme und blendet die Leiste aus. Hows läuft weiter und nimmt nichts auf, solange die Leiste ausgeblendet ist. War die Aufnahme schon pausiert, bleibt sie pausiert. Beenden fragt vorher, wenn es schon aufgenommene Schritte verwerfen würde oder Arbeit im Editor, die noch nicht gespeichert ist. Öffnest du eine `.steps`-Datei, während Hows schon läuft, fragt Hows, bevor sie Arbeit ersetzt, die noch nicht gespeichert ist, und ersetzt keine laufende Aufnahme.

### Dokumentation

- Datenschutz und README nennen die Verantwortung der aufnehmenden Person für fremde Bildschirme, Chats und das Teilen, dass die MIT-Lizenz Hows und nicht die aufgenommene Oberfläche abdeckt, und dass „kein Netzwerk“ für die laufende App gilt. Die Installer laden WebView2 von Microsoft, wenn es fehlt.

### Aufnahme (Windows)

- Klicks, Doppelklicks, Rechtsklicks und Kürzel mit Modifikatortaste werden Schritte mit Screenshot und Metadaten aus UI Automation.
- Tippen wird bis zu einer Pause ein Schritt „Gib deinen Text ein“ pro Feld; die Zeichen werden nie aufgenommen.
- Scrollen wird pro Fenster und Richtung bis zu einer Pause zusammengefasst; Touchpad-Bewegung zählt auch.
- Doppelklicks werden mit der Doppelklick-Zeit und dem Abstand des Systems erkannt.
- Tastenkürzel zum Starten und Beenden sowie zum Pausieren (`Strg+Umschalt+R`, `Strg+Umschalt+P`), einstellbar; belegt eine andere App eines beim Start, öffnen sich die Einstellungen und markieren es.
- Die eigenen Fenster von Hows werden nicht aufgenommen, auch keine Tasten, die du drückst, während ein Fenster von Hows vorne ist.
- Ein Zeichen, das du mit `AltGr` tippst, zum Beispiel `AltGr+Q` für `@` auf einer deutschen Tastatur, zählt als Tippen. Hows speichert es nicht als Tastenkürzel, und der Schritt bekommt keinen Screenshot.
- Ein Aufnahme-Kürzel braucht `Strg`, `Alt` oder `Win`. `Umschalt` darf dazu, wie bei `Strg+Umschalt+R`. `Umschalt` allein und eine Taste ohne Modifikator gelten nicht, und das bisherige Kürzel bleibt.
- Während der Aufnahme schrumpft das Hauptfenster zu einer kleinen Aufnahmeleiste oben in der Mitte des Bildschirms. Sie bleibt im Vordergrund, lässt sich ziehen und fehlt auf jedem Screenshot, auch auf denen der Schritte. Nach dem Beenden kehrt das Fenster an seine vorige Größe und Position zurück.
- Kann Hows keine Maus- und Tastatureingaben empfangen, startet die Aufnahme nicht mehr zum Schein. Hows bleibt in der Bibliothek oder im Editor und sagt in der Sprache der Oberfläche, was zu tun ist.
- Der Screenshot eines Schritts entsteht um den Mausdruck herum. Hows reicht den Druck an seinen Recorder weiter und nimmt den Screenshot direkt danach auf, er kann also schon zeigen, wie die App auf den Klick reagiert.

### Bearbeiten

- Storyboard-Editor mit Filmstreifen: umsortieren per Ziehen oder mit Alt+Pfeil nach oben und Alt+Pfeil nach unten, Schritte bearbeiten oder löschen, Anleitung umbenennen.
- Beim Öffnen einer Anleitung lädt Hows den Screenshot des ausgewählten Schritts und die Bilder im Filmstreifen, sobald sie ins Bild kommen, statt jeden Screenshot auf einmal.
- Zerstörungsfreie Markierungen: Rechteck, Pfeil, Kreis, Stift, Textmarker, Unschärfe, Text, mit Farbvorlagen und Rückgängig.
- Bibliothek mit zuletzt geöffneten Anleitungen; `.steps`-Dateien per Doppelklick oder aus der App öffnen.
- Jede zuletzt geöffnete Anleitung zeigt ihren ersten Screenshot, den Titel, die Zahl der Schritte und das Aufnahmedatum. Hows liest dafür nur die Daten der Anleitung und diesen einen Screenshot, damit die Bibliothek auch bei großen Dateien schnell bleibt. Verschobene oder gelöschte Dateien sind als fehlend markiert und verschwinden aus der Liste, wenn du sie anklickst; beschädigte Dateien sind als nicht lesbar markiert.
- Die Werkzeugleiste zum Markieren bleibt am unteren Fensterrand, wenn der Screenshot höher ist als das Fenster.
- Bevor Hows Änderungen verwirft oder alle Markierungen entfernt, fragt es in einem eigenen Dialog in der Sprache der Oberfläche. **Abbrechen** hat den Fokus, also behält `Enter` deine Arbeit.
- Eine gespeicherte Anleitung fragt beim Zurückgehen nicht mehr, ob du Änderungen verwerfen willst, egal von wo aus sie gespeichert wurde.
- Änderungen, die noch nicht gespeichert sind, gehen nicht mehr ohne Rückfrage verloren. Hows fragt auch, bevor eine neue Aufnahme die Anleitung ersetzt, egal ob sie in der Bibliothek, per Tastenkürzel oder im Infobereich startet, und bevor **Hows beenden** im Infobereich die App schließt. Pausieren, Fortsetzen und Stoppen fragen nie. Ein Schritttext, den du unverändert verlässt, zählt nicht als Änderung.
- Editor und Exporte zeichnen Markierungen mit denselben Standardwerten: Farben, Deckkraft des Textmarkers, Stiftbreite, Textgröße und Textfeld, Pfeilspitzen, den hellen Rand hinter exportiertem Text und die Blockgröße des Weichzeichners. Ein exportierter Pfeil hat dieselbe Form und Größe wie der Pfeil im Editor, und eine Markierung mit beschädigter Farbe erscheint in der Standardfarbe für Markierungen.

### Export

- HTML (eine eigenständige Datei) und PDF (A4 oder US Letter) aus der App; Markdown mit Bildern und JSON von der Kommandozeile.
- Anleitungsmodus zum Teilen, Fehlerberichtsmodus mit Zeitstempeln, Elementangaben und Betriebssystem (Kommandozeile).
- Das Exportblatt zeigt den Zielordner vor dem Export und warnt, wenn OneDrive ihn synchronisiert.
- HTML- und PDF-Exporte nutzen Stil und Akzentfarbe aus den Einstellungen, mit nummerierten Schrittmarken und einer Akzentlinie unter dem Titel. HTML folgt am Bildschirm dem dunklen Modus der Lesenden und druckt in der hellen Palette.
- Die Schrift Atkinson Hyperlegible Next ist in HTML- und PDF-Exporte eingebettet, damit sie ohne Internetverbindung gleich aussehen. PDFs zeigen jetzt Umlaute, Akzente und Buchstaben wie das polnische ł. Ein Pfeil wird als Bindestrich und Größer-als-Zeichen geschrieben, weil die Schrift kein Pfeilzeichen hat.
- Eine kurze Zeile „Erstellt mit Hows“ in der Sprache der Anleitung, am Ende von HTML und Markdown und in jeder PDF-Fußzeile. Sie ist standardmäßig an und hat keinen Link.
- Speichern und Exportieren ersetzen nie eine vorhandene Datei, in der App und auf der Kommandozeile. Ein belegter Name bekommt eine Nummer, etwa `Anleitung (2).steps`, und ein Markdown-Export bekommt immer einen neuen Ordner. Eine `.steps`-Datei entsteht erst als Temp-Datei, deshalb lässt ein fehlgeschlagenes Speichern eine vorhandene Datei intakt.
- Ein Dateiname, den Windows reserviert, etwa `CON` oder `nul.pdf`, bekommt an diesem reservierten Teil ein `_`, damit der Export geschrieben werden kann.

### Kommandozeile

- `steps-cli export`, `demo` und `from-script`, um Anleitungen ohne Desktop aus JSON zu bauen.
- `steps-cli export` nimmt `--brand`, `--accent` und `--no-credit` für das Aussehen von HTML, PDF und Markdown.
- `steps-cli` spricht alle sieben Sprachen der App. Es nimmt `--lang`, sonst die Anzeigesprache des Systems, sonst Englisch. Die Hilfe nennt den Standardstil aus der Liste der Stile, und `from-script` gibt neuen Anleitungen dieselbe Sprache, wenn das Skript keine setzt. Die Schrittbeschriftungen aus `from-script` folgen der Sprache der Anleitung, auch beim französischen Abstand.
- `--format md` funktioniert als Kurzform von `markdown`, Hilfe und Fehlermeldungen nennen sie.

### Dateiformat

- `.steps`-Schema-Version 1, beschrieben in [Das `.steps`-Dateiformat](docs/de/file-format.md).
- Beim Öffnen wird eine `.steps`-Datei gegen dokumentierte [Grenzen](docs/de/file-format.md#grenzen) geprüft: Schritt-IDs bestehen nur aus Buchstaben, Ziffern, `-` und `_` und kommen einmal vor, Größen und Anzahlen bleiben begrenzt, Markierungen bleiben nah am Bild. Eine Datei, die eine Grenze verletzt, wird als beschädigt gemeldet und nicht teilweise geöffnet. Ein Markdown-Export schreibt nie außerhalb seines Ordners. Exporte zeichnen eine Markierung nur, soweit sie im Bild liegt, deshalb hält ein Kreis, der weit über den Rand eines sehr breiten Screenshots reicht, den Export nicht mehr auf. Ein Stiftstrich, der weit länger ist, als jemand von Hand zeichnet, endet früher, statt den Export aufzuhalten.
- Vor dem Speichern prüft Hows einen Guide gegen dieselben Grenzen, deshalb öffnet dieselbe Version jede `.steps`-Datei wieder, die sie schreibt. Ein Guide, der eine Grenze verletzt, wird nicht gespeichert, und eine vorhandene Datei bleibt erhalten.
- Zwei Schritt-IDs, die sich nur in der ASCII-Großschreibung unterscheiden, gelten als dieselbe ID, weil die Bildnamen einer Anleitung unter Windows in einem Ordner liegen.

### Oberfläche

- Oberfläche auf Englisch, Deutsch, Französisch, Spanisch, Italienisch, Portugiesisch und Niederländisch, mit hellem und dunklem Farbschema. Die deutsche Oberfläche nutzt das „du“.
- Klare Sprache überall: keine internen Begriffe, keine Pfeile aus Textzeichen, jede Meldung sagt, was passiert ist und was als Nächstes zu tun ist. Ein Test hält die Texte jeder Sprache an diese Regeln.
- Schritttexte und vorgeschlagene Titel nutzen den lesbaren Namen der App aus ihren Versionsinformationen, zum Beispiel „Windows PowerShell“ statt „powershell“.
- Schritttexte, vorgeschlagene Titel und Beschriftungen der Exporte auf Englisch, Deutsch, Französisch, Spanisch, Italienisch, Portugiesisch und Niederländisch. Schritttexte sind als Anweisungen formuliert, zum Beispiel *Klicke auf „Speichern“ in „Editor“*.
- Die Sprache steht zu Beginn auf **Wie Windows** und folgt der Windows-Anzeigesprache, sonst gilt Englisch. Eine Sprache, die du wählst, bleibt fest.
- Ein Sprachwechsel gilt sofort für das Menü im Infobereich und die Dateidialoge und für Schritttexte ab der nächsten Aufnahme, ohne Neustart.
- Die Bibliothek und die Aufnahmeleiste zeigen die eingestellten Kürzel für Aufnahme und Pause. In der Leiste steht jedes Kürzel auf seiner Schaltfläche, damit die Leiste in jeder Sprache kurz bleibt.
- Die Bibliothek lädt ihre zuletzt geöffneten Anleitungen bei jedem Öffnen neu, damit eine eben gespeicherte Anleitung dabei ist.
- Screenreader kündigen die Schritte im Filmstreifen als verschiebbar an, in der Sprache der Oberfläche.
- Fehlermeldungen erscheinen in der Sprache der Oberfläche und sagen, was du als Nächstes tun kannst, zum Beispiel beim Speichern, Exportieren, bei der Ordnerwahl oder beim Öffnen einer Anleitung aus einer neueren Hows-Version. Technische Details bleiben aus der Oberfläche heraus. Ein Test prüft, dass jeder Fehler in jeder Sprache eine Meldung hat.
- Französische Texte setzen ein geschütztes Leerzeichen vor `:`, `;`, `!` und `?` und innerhalb von « », auch in den Beschriftungen der Fehlerberichtsexporte und in den Meldungen von `steps-cli`. Der Abstand ist in der App, in den Exporten und auf der Kommandozeile sichtbar. Ein Test prüft das für Oberfläche, Exporttexte und Kommandozeile, ein weiterer, dass die mitgelieferte Schrift jedes Zeichen jeder Sprache darstellen kann.

### Design

- Drei Stilvorlagen, Salbei (Standard), Tinte und Glut, jeweils mit heller und dunkler Palette. Jede Textfarbe erfüllt den Kontrast nach WCAG AA, geprüft von einem Test.
- Eine eigene Akzentfarbe auf jeder Vorlage. Die App dunkelt sie ab oder hellt sie auf, bis Text darauf lesbar bleibt.
- Die mitgelieferte Schrift Atkinson Hyperlegible Next, gemacht für Lesbarkeit, sodass sich `Il1` und `0O` unterscheiden. Keine Schrift wird aus dem Internet geladen.
- Die App meldet sich beim Start nicht mehr bei Google Fonts. Ein übrig gebliebener Link in der Seite fragte dort die Schrift Inter an, die die App gar nicht nutzt. Ein Test und eine Prüfung nach jedem Build schlagen jetzt fehl, wenn die Seite, ihre Stile oder die gebaute App eine Webadresse nennen, die geladen werden könnte, auch Adressen, die mit `//` beginnen.
- Eine Content Security Policy lässt das App-Fenster nur die mitgelieferten Skripte, Stile und die Schrift, die Screenshots der Schritte und die eigene Verbindung zur App laden und lehnt alles andere ab, auch eingebettete Skripte und alles aus dem Internet.
- Symbole statt Textzeichen für Werkzeuge, Schaltflächen und die Aufnahmeleiste.
- Ein sichtbarer Fokusrahmen für die Tastaturbedienung, leere Zustände mit einem Hinweis auf den nächsten Schritt und Fehler in Rot mit Symbol.
- Kontrollkästchen, Optionsfelder und Bildlaufleisten folgen dem Akzent und dem Farbschema.
- Abstände, Schriftgrößen und die Rundungen der Ecken kommen aus gemeinsamen Skalen, damit dieselbe Stufe überall gleich aussieht. Ein Test meldet jeden neuen Wert außerhalb der Skalen.

### Einstellungen

- Papierformat für PDF (A4 oder US Letter) und die Farbe neuer Markierungen stehen bei den übrigen Einstellungen für Export und Darstellung.
- Stilvorlage und Akzentfarbe unter Darstellung, jeweils mit eigener Schaltfläche zum Zurücksetzen.
- Ein Kontrollkästchen unter Export schaltet die Zeile „Erstellt mit Hows“ an oder aus, mit eigener Schaltfläche zum Zurücksetzen.
- Ein Bereich „Erweitert“ enthält die selten gebrauchten Werte: die Tipp- und Scroll-Pausen, nach denen ein neuer Schritt beginnt, die Zahl der zuletzt geöffneten Anleitungen, den Seitenrand für PDF, die Linienstärke neuer Markierungen sowie Vorlagen für Titel (`{app}`, `{date}`, `{time}`) und Dateinamen (`{title}`, `{app}`, `{date}`, `{time}`). Die Vorlagen zeigen sofort ein Beispiel. Datum und Uhrzeit folgen der Zeitzone deines PCs, und `{time}` steht als `14.05` da, damit es in Dateinamen funktioniert.
- Jeder geänderte Wert hat seine eigene Schaltfläche zum Zurücksetzen, und eine Schaltfläche stellt alle erweiterten Werte wieder her. Werte außerhalb des erlaubten Bereichs werden begrenzt, in der App und beim Laden der Einstellungsdatei.
- `settings.json` wird erst ersetzt, wenn die neue Datei vollständig ist, deshalb bleibt bei einem fehlgeschlagenen Speichern die bisherige Einstellung nutzbar.
- Zahlen in den Einstellungen nutzen das Dezimalzeichen der Oberflächensprache, zum Beispiel 1.5 auf Englisch und 1,5 auf Deutsch. Beim Tippen gelten beide Zeichen, und die Pfeiltasten ändern den Wert schrittweise.
- Alle Bedienelemente der Einstellungen sind gleich breit. Ein langer Exportordner wird um ganze Ordner gekürzt, zum Beispiel `C:\…\Documents`, und zeigt den vollen Pfad, wenn du darauf zeigst.

### Dokumentation

- Benutzerhandbuch auf Englisch und Deutsch: Installation, SmartScreen-Warnung, Aufnahme, Bearbeiten, Export, Einstellungen, Tastenkürzel, Datenschutz, Kommandozeile und Probleme lösen. README, Anleitung zum Mitmachen, Sicherheitsrichtlinie und Änderungsprotokoll gibt es in beiden Sprachen, und jede Seite verlinkt ihre Übersetzung.
- CI prüft, dass jede Seite in beiden Sprachen existiert und relative Links und Anker auflösen, und prüft die Rechtschreibung beider Sprachen.
- CI gleicht die dokumentierten Standards, Bereiche, Tastenkürzel, Formate, Beschriftungen und Meldungen mit dem Code ab. Die Seiten markieren diese Werte, und beide Sprachen müssen dieselben markieren.
- Das Handbuch nennt jetzt jede Fehlermeldung, die die App zeigen kann, auch die für ein Fenster zum Auswählen, das nicht aufgeht, und zitiert die Startmeldung für ein Tastenkürzel, das eine andere App belegt. Nach dem Löschen des letzten Schritts ist der vorige ausgewählt, wie in der App. Die Datenschutzseite sagt, welche Angaben eine `.steps`-Datei enthalten kann und wann sie fehlen, und die SmartScreen-Seite verspricht nicht mehr, dass signierte Builds die Warnung beenden.
- Die Datenschutzseite, die README und die Seiten zu Export, Kommandozeile und Dateiformat beschreiben, was Hows aufnimmt, so wie der Code es tut. Getippte Zeichen werden nicht als Tastatureingabe gespeichert. Exporte im Anleitungsmodus lassen die getrennten technischen Felder weg, aber ein Schritttext kann eine sichtbare Schaltfläche oder ein Feld nennen und den Fenster- oder Seitentitel. Aufnahmen aus der App enthalten vorerst keine Windows-Version, ein Fehlerbericht nennt sie also nicht immer.
- Das Formular für einen falschen Schritttext bietet alle sieben Sprachen der Anleitungen an.

### Build

- CI und Release-Builds laufen auf festen Runner-Images, Ubuntu 24.04 und Windows Server 2025, damit ein Wechsel des Standard-Images bei GitHub kein Release bricht.
- `THIRD-PARTY-NOTICES.md` nennt die Schrift, die Pakete der Oberfläche und die Rust-Crates, die in der App stecken, mit ihren Lizenztexten. Die Installer für NSIS und MSI legen diese Datei und `LICENSE` neben `hows.exe`.
- Die portable Verteilung ist `Hows_*_x64-portable.zip` mit `hows.exe`, `LICENSE` und `THIRD-PARTY-NOTICES.md`. Der Release-Workflow lädt `hows.exe` nicht allein hoch.
- Das MSI reist mit `wix-UIExtension-wix3141rtm.zip`, der Quelle der WiX-3.14.1-Oberfläche am Commit `b40e9a32c24033e11b77baf2c91a704382f898ed` und ihrer Microsoft Reciprocal License. Der NSIS-Hinweis nennt das Quellenarchiv `nsis-3.11-src.tar.bz2`.
- `THIRD-PARTY-NOTICES.md` nennt auch die Pakete, die `npm ls --omit=dev` für die Oberfläche installiert, einschließlich der Compiler-Abhängigkeiten von Svelte. Diese Compiler-Pakete stehen nicht in `hows.exe`.
- Jede GitHub Action in den Workflows ist auf einen Commit-SHA festgelegt.
- Zum Bauen braucht Hows Rust 1.88 oder neuer, die älteste Version, mit der die festgelegten Abhängigkeiten bauen.
