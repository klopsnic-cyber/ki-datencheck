# KI-Datencheck

Prüft Texte und Dateien, **bevor** sie in eine KI (ChatGPT, Copilot, Claude …) gehen: Ampel mit Begründung, markierte Fundstellen, anonymisierte Fassung zum Kopieren. Alles läuft lokal; nichts verlässt den Rechner.

Erkennt (Regeln + Prüfziffern, `src-tauri/src/detect.rs`):
- Zugangsdaten: Passwörter, private Schlüssel, API-Tokens, Passwörter in URLs
- Kennungen: IBAN (mod 97), Kreditkarte (Luhn), KVNR (Prüfziffer), Steuer-ID (Prüfziffer), SV-Nummer, Patienten-/Fall-/Kundennummern
- Personen: Namen (Anrede, „Patient:“, häufige Vornamen), Geburtsdatum
- Kontakt: E-Mail, Telefon, Straße + Hausnummer, PLZ + Ort
- Gesundheit (Art. 9 DSGVO): Fachbegriffe, Diagnosen, Medikamente, ICD-10
- Weitere besondere Kategorien: Religion, Herkunft, Gewerkschaft, Vorstrafen …
- Technik: IP-Adressen

Bewertung: Rot bei Zugangsdaten, eindeutigen Kennungen oder Gesundheits-/Art.-9-Daten mit Personenbezug; Gelb bei sonstigem Personenbezug oder sensiblen Themen ohne Namen; sonst Grün.

Dateien: PDF, docx/xlsx/pptx, odt/ods/odp, alle Textformate (txt, csv, json, eml, log …). Alte .doc/.xls nicht.

## Entwickeln
```
npm install
npm run tauri dev                    # KD_DEMO=1 lädt den Beispieltext (nur Dev)
cd src-tauri && cargo test
cargo run --example check -- a.pdf   # Dateien auf der Kommandozeile prüfen
```

## Release
`./release.sh 0.1.1 "Was ist neu"` → GitHub Actions baut macOS (universal) und Windows samt `latest.json`; die Apps bieten das Update selbst an (Start + alle 4 h).
Signaturschlüssel: `~/.tauri/ki-datencheck.key` (+ `.passwort`), als Repo-Secrets hinterlegt. Niemals ausgeben oder committen.
