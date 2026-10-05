[English](SECURITY.md) | Deutsch

# Sicherheitsrichtlinie

## Eine Schwachstelle melden

Bitte melde Sicherheitsprobleme vertraulich über GitHub: **Security → Report a vulnerability** in diesem Repository. Eröffne kein öffentliches Issue.

Schreib dazu, was du gefunden hast, wie man es nachstellt und welche Version oder welchen Commit du getestet hast. Du bekommst innerhalb einer Woche eine Antwort. Sobald ein Fix veröffentlicht ist, wird die Meldung mit Nennung deines Namens veröffentlicht, außer du möchtest das nicht.

## Was besonders zählt

Für Hows besonders wichtig:

- alles, was getippten Text speichert oder exportiert, was die Aufnahme nie tun darf;
- Dateien, die außerhalb des vom Nutzer gewählten Ordners geschrieben werden, außer im Standardordner, den Hows nimmt, wenn der gewählte nicht nutzbar ist;
- präparierte `.steps`-Dateien, die beim Öffnen die App abstürzen lassen, Code ausführen oder Dateien schreiben;
- Netzwerkverkehr jeder Art, da die App keinen erzeugen soll.

Was Hows bewusst aufnimmt und speichert, beschreibt [Datenschutz](docs/de/privacy.md).

## Unterstützte Versionen

Nur das neueste Release bekommt Sicherheitsupdates.
