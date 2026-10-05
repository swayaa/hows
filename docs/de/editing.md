[English](../en/editing.md) | Deutsch

# Bearbeiten und markieren

Wenn du eine Aufnahme beendest, öffnet sich der Editor. Oben, in einer Zeile, siehst du den Titel der Anleitung, <!-- fact:ui.annotateMode -->**Markieren**<!-- /fact --> und **Exportieren**. Darunter liegt der Streifen <!-- fact:ui.filmstrip -->**Schritte**<!-- /fact --> mit einer Karte pro Schritt. Schritttext und <!-- fact:ui.deleteStep -->**Schritt löschen**<!-- /fact --> stehen über dem Screenshot.

## Der Anleitung einen Namen geben

Hows schlägt einen Titel aus der benutzten App und dem Datum der Aufnahme vor, zum Beispiel *Anleitung für Editor (2026-09-30)*. Das Titelfeld ist markiert, du kannst also sofort einen besseren Namen tippen. Das Muster änderst du unter [Einstellungen](settings.md#erweitert).

## Schritte auswählen und umsortieren

- Klick im Streifen **Schritte** auf eine Karte, um diesen Schritt zu sehen.
- Hat der Streifen den Fokus, wählen Pfeil nach oben und Pfeil nach unten den vorigen oder nächsten Schritt. Pos1 und Ende wählen den ersten oder letzten.
- Um einen Schritt zu verschieben, zieh seinen Griff (<!-- fact:ui.gripLabel -->**Schritt verschieben**<!-- /fact -->) an die neue Stelle. Hat der Griff den Fokus, schiebt Alt+Pfeil nach oben oder Alt+Pfeil nach links den Schritt eine Stelle nach vorn. Alt+Pfeil nach unten oder Alt+Pfeil nach rechts schiebt ihn eine Stelle nach hinten.

## Den Schritttext ändern

Bearbeite den Text im Feld über dem Screenshot. `Enter` übernimmt den Text und verlässt das Feld. `Umschalt+Enter` beginnt eine neue Zeile. Eine Zeile unter dem Feld sagt das. Das Feld auf anderem Weg zu verlassen übernimmt die Änderung ebenfalls. Ein Hinweis zeigt dann, dass du den Text geändert hast; leere das Feld, um den erzeugten Text zurückzubekommen.

## Einen Schritt löschen

Wähl den Schritt aus und klick auf <!-- fact:ui.deleteStep -->**Schritt löschen**<!-- /fact -->. Danach ist der nächste Schritt ausgewählt, beim letzten Schritt der vorige.

## Screenshots markieren

Klick auf **Markieren**, um auf dem Screenshot des ausgewählten Schritts zu zeichnen. Die Werkzeugleiste bleibt am unteren Fensterrand, auch wenn der Screenshot höher ist als das Fenster. Klick auf <!-- fact:ui.annotateDone -->**Fertig**<!-- /fact -->, wenn du fertig bist.

| Werkzeug | Wofür |
|---|---|
| <!-- fact:ui.tools.rect -->Rechteck<!-- /fact --> | Eine Schaltfläche oder einen Bereich einrahmen. |
| <!-- fact:ui.tools.arrow -->Pfeil<!-- /fact --> | Auf etwas zeigen. |
| <!-- fact:ui.tools.circle -->Kreis<!-- /fact --> | Ein Detail einkreisen. |
| <!-- fact:ui.tools.pen -->Stift<!-- /fact --> | Frei zeichnen. |
| <!-- fact:ui.tools.highlight -->Textmarker<!-- /fact --> | Einen Bereich mit durchscheinender Farbe hervorheben. |
| <!-- fact:ui.tools.blur -->Unschärfe<!-- /fact --> | Einen Bereich unlesbar machen, zum Beispiel Namen oder Nummern. |
| <!-- fact:ui.tools.text -->Text<!-- /fact --> | Eine kurze Beschriftung setzen. `Enter` setzt sie, `Esc` bricht ab. |

- Die Schaltfläche <!-- fact:ui.annotateColor -->**Farbe**<!-- /fact --> legt die Farbe der nächsten Markierung fest und merkt sie sich. Neue Markierungen starten mit <!-- fact:settings.annotation_color -->`#E11D48`<!-- /fact -->, bis du eine andere Farbe wählst. Die Linienstärke kommt aus den [erweiterten Einstellungen](settings.md#erweitert).
- Klick auf eine Markierung, um sie auszuwählen. `Entf` oder `Rücktaste` entfernt die ausgewählte Markierung, `Esc` hebt die Auswahl auf.
- Ein Klick auf <!-- fact:ui.annotateUndo -->**Rückgängig**<!-- /fact --> nimmt die letzte Änderung zurück. <!-- fact:ui.annotateClear -->**Alle Markierungen entfernen**<!-- /fact --> leert den Schritt, nachdem du mit <!-- fact:ui.annotateClearAction -->**Alle entfernen**<!-- /fact --> bestätigt hast.

Markierungen verändern nie den ursprünglichen Screenshot. Hows speichert sie getrennt und zeichnet sie erst in Exporte ein. Unscharfe Bereiche sind in jedem Export verpixelt.

## Einen Screenshot zuschneiden

Klick auf **Markieren**, dann in dieser Leiste auf <!-- fact:ui.crop -->**Zuschneiden**<!-- /fact -->. Zieh eine Ecke des Rahmens. <!-- fact:ui.cropApply -->**Zuschnitt übernehmen**<!-- /fact --> behält dieses Fenster. `Enter` tut dasselbe, solange der Rahmen offen ist, und `Esc` bricht ab. Das Originalbild bleibt in der Datei. <!-- fact:ui.cropReset -->**Ganzes Bild zeigen**<!-- /fact --> holt die verdeckten Ränder zurück. Exporte zeigen das Fenster.

## Deine Arbeit sichern

Hows speichert nicht automatisch. Um eine bearbeitbare Fassung zu behalten, exportierst du die Anleitung als **Hows-Datei (.steps)**, siehe [Exportieren und teilen](export.md#hows-datei-steps). Gehst du mit ungespeicherten Änderungen zurück zur Bibliothek, öffnest eine andere Anleitung, startest eine neue Aufnahme, drückst `Alt+F4` oder wählst am Hows-Symbol im Infobereich <!-- fact:i18n.tray.quit -->**Hows beenden**<!-- /fact -->, fragt Hows vorher, ob du sie verwerfen willst. Beenden fragt auch, wenn eine Aufnahme schon Schritte hat. Das gilt auch für eine Aufnahme, die du per Tastenkürzel oder im Infobereich startest. <!-- fact:ui.discardAction -->**Verwerfen**<!-- /fact --> verwirft die Änderungen; <!-- fact:ui.confirmCancel -->**Abbrechen**<!-- /fact --> lässt dich im Editor. **Abbrechen** hat den Fokus, also behält `Enter` deine Arbeit.

## Eine gespeicherte Anleitung öffnen

- Klick in der Bibliothek auf <!-- fact:ui.open -->**Öffnen…**<!-- /fact --> und wähle eine `.steps`-Datei, oder klick unter <!-- fact:ui.recent -->**Zuletzt geöffnet**<!-- /fact --> auf eine Anleitung. Jede Karte zeigt den ersten Screenshot, den Titel, die Zahl der Schritte und das Aufnahmedatum.
- Anleitungen, deren Datei verschoben oder gelöscht wurde, sind mit <!-- fact:ui.missing -->**Datei fehlt**<!-- /fact --> markiert. Klickst du eine an, sagt Hows das und nimmt sie aus der Liste. Dateien, die keine lesbare Anleitung sind, sind mit <!-- fact:ui.recentUnreadable -->**Nicht lesbar**<!-- /fact --> markiert.
- Hast du einen Installer benutzt, reicht ein Doppelklick auf eine `.steps`-Datei im Datei-Explorer. Während Hows aufnimmt oder pausiert, zeigt der Doppelklick die Aufnahmeleiste und lässt die Aufnahme so, wie sie ist. Ist Hows schon offen und die Anleitung hat Änderungen, die noch nicht gespeichert sind, fragt Hows, bevor es die Datei öffnet.

Die Anleitung öffnet sich im Editor, und du kannst sie ändern und erneut exportieren.

---

[Zur Übersicht](README.md) · Weiter: [Exportieren und teilen](export.md)
