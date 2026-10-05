[English](../en/recording.md) | Deutsch

# Eine Anleitung aufnehmen

## Aufnahme starten

Du hast drei Wege:

- Klick in der Bibliothek auf <!-- fact:ui.newCapture -->**Neue Aufnahme**<!-- /fact -->.
- Drück das Aufnahme-Tastenkürzel, standardmäßig <!-- fact:settings.hotkey keys -->`Strg+Umschalt+R`<!-- /fact -->.
- Klick auf das Hows-Symbol im Infobereich und wähle <!-- fact:i18n.tray.toggle -->**Aufnahme starten oder beenden**<!-- /fact -->.

Dann arbeitest du wie gewohnt. Während Hows aufnimmt, schrumpft sein Fenster zu einer kleinen Aufnahmeleiste oben in der Mitte des Bildschirms. Die Leiste zeigt einen pulsierenden roten Punkt, die Zahl der bisherigen Schritte und die Schaltflächen **Pause** und **Stopp**, jeweils mit ihrem Tastenkürzel. Während du pausierst, steht dort <!-- fact:ui.capturePaused -->**Pausiert**<!-- /fact -->. Sie bleibt über anderen Fenstern, und du kannst sie woandershin ziehen. Die eigenen Fenster nimmt Hows nie auf. Ein Klick auf die Leiste ergibt keinen Schritt, ebenso wenig Tasten, die du drückst, während ein Fenster von Hows vorne ist. Die Leiste erscheint auf keinem Screenshot. Wenn du beendest, kehrt das Fenster an seine vorige Größe und Position zurück.

Windows lässt die Leiste während der Aufnahme aus jeder Bildschirmaufnahme weg. Das gilt auch für deine eigenen Screenshots, Bildschirmvideos und Bildschirmfreigaben, zum Beispiel in einem Videoanruf. Die anderen im Anruf sehen die Leiste nicht, du selbst schon.

## Was zu einem Schritt wird

| Was du tust | Daraus wird | Screenshot |
|---|---|---|
| Klicken, doppelklicken oder rechtsklicken | Je ein Schritt, zum Beispiel *Klicke auf „Speichern“ in „Editor“* | Ja |
| Ein Tastenkürzel mit `Strg`, `Alt` oder `Win` drücken, zum Beispiel `Strg+S` | Ein Schritt, zum Beispiel *Drücke Strg+S* | Ja |
| Mit Mausrad oder Touchpad scrollen | Ein Schritt pro Fenster und Scrollrichtung, zum Beispiel *Scrolle 3 in „Dokumente“ nach unten* | Ja |
| Text tippen | Ein Schritt pro Feld bis zu einer Pause, zum Beispiel *Gib deinen Text in „Suche“ ein* | Nein |

Im Einzelnen:

- **Doppelklicks** erkennt Hows mit der Doppelklick-Geschwindigkeit und dem Abstand, die in Windows eingestellt sind.
- **Scrollen** im selben Fenster und in dieselbe Richtung bleibt ein Schritt, bis du eine Pause machst. Wie lang die Pause ist, stellst du ein (standardmäßig <!-- fact:settings.scroll_pause_ms seconds -->1,5<!-- /fact --> Sekunden). Der Satz nennt dieses Fenster und wie viele Rasterstufen du gescrollt hast. Weitere Bewegung in der Pause erhöht nur diese Zahl. Hows schaut nur beim Start des Schritts nach, was unter dem Zeiger liegt, nicht bei jeder weiteren Bewegung.
- **Tippen** bleibt ein Schritt, bis du länger als die Tipp-Pause wartest (standardmäßig <!-- fact:settings.typing_pause_ms seconds -->3<!-- /fact --> Sekunden). Hows speichert nie, was du tippst, nur dass du getippt hast und wo. Tipp-Schritte bekommen keinen Screenshot, weil er den Text zeigen würde. Der Klick, mit dem du das Feld ausgewählt hast, zeigt es meistens.
- `Umschalt` allein ergibt kein Tastenkürzel. `Umschalt+A` zählt als Tippen, ebenso ein Zeichen, das du mit `AltGr` tippst, zum Beispiel `AltGr+Q` für `@` auf einer deutschen Tastatur.
- Jeder Screenshot zeigt den ganzen Bildschirm, auf dem der Mauszeiger steht.

Texte für Klick, Rechtsklick, Doppelklick und Tippen nennen das Steuerelement und, wenn Hows es kennt, das Fenster. Ein Scroll-Text nennt das Fenster und die Zahl der Rasterstufen. Hows schreibt sie in der Sprache, die beim Start der Aufnahme unter [Einstellungen](settings.md#darstellung) gewählt ist.

## Pausieren und fortsetzen

Klick in der Leiste auf **Pause**, drück das Pause-Tastenkürzel (standardmäßig <!-- fact:settings.pause_hotkey keys -->`Strg+Umschalt+P`<!-- /fact -->) oder wähle am Symbol im Infobereich <!-- fact:i18n.tray.pause -->**Pausieren oder fortsetzen**<!-- /fact -->. Solange die Aufnahme pausiert ist, nimmt Hows nichts auf. Klick auf **Fortsetzen** oder drück das Kürzel noch einmal, um weiterzumachen.

`Alt+F4` beendet Hows. Jedes andere Schließen der Aufnahmeleiste, zum Beispiel über die Taskleiste, pausiert die Aufnahme und blendet die Leiste aus. Hows läuft weiter und nimmt nichts auf, solange die Leiste ausgeblendet ist. War die Aufnahme schon pausiert, bleibt sie pausiert. Würde das Beenden schon aufgenommene Schritte verwerfen oder Arbeit im Editor, die noch nicht gespeichert ist, fragt Hows vorher. Siehe [Deine Arbeit sichern](editing.md#deine-arbeit-sichern).

## Beenden

Klick auf **Stopp**, drück das Aufnahme-Tastenkürzel noch einmal oder wähle am Symbol im Infobereich **Aufnahme starten oder beenden**. Der Editor öffnet sich mit allen Schritten. Weiter geht es mit [Bearbeiten und markieren](editing.md).

Hast du keinen Schritt aufgenommen, sagt dir der Editor das und bietet eine neue Aufnahme an.

## Tipps für saubere Anleitungen

- Schließe Fenster mit privaten Daten, bevor du startest, oder pausiere, solange sie zu sehen sind.
- Arbeite so ruhig, dass jeder Klick das Element trifft, das du meinst.
- Screenshots zeigen alles, was auf dem Bildschirm ist. Prüfe sie vor dem Teilen und nutze das Werkzeug Unschärfe für heikle Stellen. Siehe [Datenschutz](privacy.md).
- Hows fragt andere Personen vor der Aufnahme nicht um Einwilligung, und Windows zeigt für diese Hooks keinen Dialog zur Bildschirmaufnahme. Den Bildschirm, den Arbeitsplatz, einen Anruf oder einen Chat einer anderen Person aufzunehmen oder zu teilen, liegt in deiner Verantwortung. Siehe [Deine Verantwortung beim Aufnehmen und Teilen](privacy.md#deine-verantwortung-beim-aufnehmen-und-teilen).

## Grenzen

- Aufnehmen funktioniert nur unter Windows.
- Apps, die Windows wenig über ihre Bedienelemente mitteilen (manche Spiele, Remotedesktops, selbst gezeichnete Oberflächen), bekommen allgemeine Schritttexte, die nur das Fenster nennen, nicht die Schaltfläche. Im Editor kannst du sie umschreiben.

---

[Zur Übersicht](README.md) · Weiter: [Bearbeiten und markieren](editing.md)
