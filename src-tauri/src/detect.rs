// Erkennung personenbezogener und sensibler Daten – nur Regeln und Prüfziffern, alles lokal.
// Bewusst eher streng: lieber einmal zu viel markieren als einen Patientennamen in die KI geben.
use regex::Regex;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::LazyLock;

/// Gruppen bestimmen die Bewertung, Kinds die Anzeige.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Group {
    Geheim,
    Finanz,
    Kennung,
    Gesundheit,
    Sensibel,
    Person,
    Kontakt,
    Technik,
}

pub struct Kind {
    pub id: &'static str,
    pub label: &'static str,
    pub tag: &'static str, // Platzhalter in der anonymisierten Fassung
    pub group: Group,
    re: &'static LazyLock<Regex>,
    check: fn(&str) -> bool,
}

macro_rules! re {
    ($name:ident, $pat:expr) => {
        static $name: LazyLock<Regex> = LazyLock::new(|| Regex::new($pat).unwrap());
    };
}

const NAME: &str = r"[A-ZÄÖÜ][a-zäöüß]+(?:-[A-ZÄÖÜ][a-zäöüß]+)?";

re!(PRIVKEY, r"-----BEGIN [A-Z ]*PRIVATE KEY-----[\s\S]*?-----END [A-Z ]*PRIVATE KEY-----");
re!(APIKEY, r"\b(?:sk-[A-Za-z0-9_-]{20,}|gh[pousr]_[A-Za-z0-9]{30,}|github_pat_\w{30,}|AKIA[0-9A-Z]{16}|xox[abprs]-[A-Za-z0-9-]{10,}|AIza[0-9A-Za-z_-]{35}|eyJ[\w-]{10,}\.[\w-]{10,}\.[\w-]{10,})");
re!(PASSWORD, r"(?i)\b(?:passwort|password|passwd|pwd|kennwort|pin|zugangscode|secret|token|api[_-]?key)\s*[:=]\s*(\S{4,})");
re!(CONNSTR, r"\b[a-z][a-z0-9+.-]*://[^\s:/@]+:([^\s@/]+)@");
re!(IBAN, r"\b[A-Z]{2}\d{2}(?: ?[A-Z0-9]){11,30}\b");
re!(CARD, r"\b[3-6]\d{3}(?:[ -]?\d{4}){2}[ -]?\d{1,7}\b");
re!(KVNR, r"\b[A-Z]\d{9}\b");
re!(STEUERID, r"\b[1-9]\d{10}\b|\b[1-9]\d \d{3} \d{3} \d{3}\b");
re!(SVNR, r"\b\d{2} ?\d{6} ?[A-Z] ?\d{3}\b");
re!(IDNUMMER, r"(?i)\b(?:pat(?:ienten)?[-. ]?(?:nr|id|nummer)|fall[-]?(?:nr|nummer)|versicherten[-]?(?:nr|nummer)|kunden[-]?(?:nr|nummer)|mitglieds[-]?(?:nr|nummer)|personal[-]?(?:nr|nummer))\.?\s*:?\s*([A-Z0-9][A-Z0-9-]{3,})");
re!(BIRTH, r"(?i)\b(?:geb\.?|geboren(?:\s+am)?|geburtsdatum|geb\.-datum|dob|date of birth)\s*:?\s*(\d{1,2}\.\s?\d{1,2}\.\s?\d{2,4})");
re!(TITLE_NAME, &format!(r"\b(?:Herrn?|Frau|Hr\.|Fr\.)\s+(?:(?:Dr|Prof|med|dent)\.\s*)*({NAME}(?:\s+{NAME})?)"));
re!(LABEL_NAME, &format!(r"(?i:\b(?:patient(?:in)?|pat\.|name|nachname|vorname|versicherte[rn]?|mitarbeiter(?:in)?|klient(?:in)?|bewohner(?:in)?)\s*:\s*)({NAME}(?:,?\s+{NAME})?)"));
re!(FIRST_NAME, &format!(r"\b(?:{})\s+{NAME}\b", FIRST_NAMES));
re!(EMAIL, r"\b[\w.+-]+@[\w-]+(?:\.[\w-]+)*\.[A-Za-z]{2,}\b");
re!(PHONE, r"(?:\+|\b00)\d{2}[\s/-]?\(?0?\d{2,5}\)?[\s/-]?\d[\d\s/-]{4,12}\d|\b0\d{2,5}[\s/-]?\d[\d\s/-]{4,10}\d\b");
re!(STREET, r"\b[A-ZÄÖÜ][a-zäöüß-]*(?:straße|strasse|str\.|weg|allee|platz|gasse|ring|damm|ufer|chaussee)\s+\d{1,4}\s?[a-zA-Z]?\b");
// PLZ + Ort nur am Zeilenanfang oder direkt nach einer Straße – sonst wäre „10000 Euro“ ein Ort.
re!(PLZORT, r"(?m)(?:^[ \t]*|(?:straße|strasse|str\.|weg|allee|platz|gasse|ring|damm|ufer|chaussee)\s+\d{1,4}\s?[a-zA-Z]?,?\s+)(\d{5}\s+[A-ZÄÖÜ][a-zäöüß]+(?:[- ][A-ZÄÖÜ][a-zäöüß]+)?)");
re!(IPV4, r"\b(?:\d{1,3}\.){3}\d{1,3}\b");
re!(HEALTH, r"(?i)\b(?:diagnos\w*|befund\w*|anamnese|medikation|medikament\w*|therapie\w*|arbeitsunfähig\w*|krankschreibung|au-bescheinigung|arztbrief|op-bericht|epikrise|laborwert\w*|blutbild|röntgen\w*|mrt|pflegegrad|schwerbehindert\w*|behinderung|diabetes|depressi\w+|demenz|alzheimer|krebs|karzinom\w*|tumor\w*|metasta\w+|chemotherapie|hiv|aids|hepatitis|schwanger\w*|psychotherap\w*|psychiatr\w*|psychos\w+|suizid\w*|sucht\w*|alkoholabhängig\w*|epilepsie|schizophren\w*|borderline|adhs|autismus|asthma|hypertonie|herzinfarkt|schlaganfall|dialyse|allergie\w*|covid\w*|infektion\w*|rezept\w*|überweisung\w*zum|ibuprofen|metformin|insulin)\b");
re!(ICD, r"\b[A-TV-Z]\d{2}(?:\.\d{1,2})?[GVAZRLB]?\b");
re!(SENSIBEL, r"(?i)\b(?:religion|konfession|katholisch|evangelisch|muslim\w*|jüdisch\w*|gewerkschaft\w*|homosexu\w*|transgender|sexuelle[nr]? orientierung|ethnische[rn]? herkunft|vorstrafe\w*|strafregister|führungszeugnis|biometrisch\w*|fingerabdr\w*|parteimitglied\w*)\b");

const FIRST_NAMES: &str = "Anna|Maria|Emma|Mia|Sophie|Sophia|Hannah|Lena|Laura|Lea|Leonie|Julia|Sarah|Lisa|Katharina|Johanna|Clara|Marie|Paula|Charlotte|Emilia|Lina|Ella|Ida|Greta|Sabine|Susanne|Petra|Monika|Andrea|Claudia|Birgit|Stefanie|Nicole|Christina|Kerstin|Martina|Ursula|Renate|Ingrid|Gisela|Helga|Karin|Elke|Heike|Anja|Silke|Tanja|Melanie|Jessica|Jennifer|Nadine|Katrin|Daniela|Sandra|Manuela|Angelika|Barbara|Christine|Elisabeth|Gabriele|Brigitte|Jutta|Ruth|Doris|Lukas|Leon|Finn|Jonas|Paul|Ben|Elias|Noah|Felix|Max|Maximilian|Luca|Tim|Tom|Jan|Niklas|Moritz|David|Simon|Philipp|Alexander|Sebastian|Florian|Tobias|Daniel|Michael|Thomas|Andreas|Stefan|Christian|Markus|Martin|Frank|Peter|Klaus|Jürgen|Wolfgang|Dieter|Uwe|Ralf|Holger|Matthias|Torsten|Sven|Oliver|Dirk|Jens|Marco|Patrick|Dennis|Kevin|Marcel|Benjamin|Johannes|Julian|Fabian|Dominik|Manfred|Werner|Hans|Heinz|Günter|Horst|Helmut|Bernd|Gerhard|Karl|Josef|Rainer|Hartmut|Volker|Mohammed|Mehmet|Ali|Ahmet|Fatma|Ayse|Elif|Emre|Can|Murat|Ivan|Olga|Natalia|Piotr|Anja";

fn always(_: &str) -> bool {
    true
}

macro_rules! k {
    ($id:expr, $label:expr, $tag:expr, $g:ident, $re:ident) => {
        k!($id, $label, $tag, $g, $re, always)
    };
    ($id:expr, $label:expr, $tag:expr, $g:ident, $re:ident, $check:expr) => {
        Kind { id: $id, label: $label, tag: $tag, group: Group::$g, re: &$re, check: $check }
    };
}

/// Reihenfolge = Vorrang bei Überschneidungen.
pub static KINDS: LazyLock<Vec<Kind>> = LazyLock::new(|| {
    vec![
        k!("privkey", "Privater Schlüssel", "SCHLÜSSEL", Geheim, PRIVKEY),
        k!("apikey", "API-Schlüssel / Token", "TOKEN", Geheim, APIKEY),
        k!("password", "Passwort / Zugangscode", "PASSWORT", Geheim, PASSWORD),
        k!("connstr", "Passwort in Verbindungsadresse", "PASSWORT", Geheim, CONNSTR),
        k!("iban", "IBAN", "IBAN", Finanz, IBAN, iban_ok),
        k!("card", "Kreditkartennummer", "KARTE", Finanz, CARD, luhn_ok),
        k!("kvnr", "Krankenversichertennummer", "KVNR", Kennung, KVNR, kvnr_ok),
        k!("svnr", "Sozialversicherungsnummer", "SVNR", Kennung, SVNR),
        k!("steuerid", "Steuer-ID", "STEUER-ID", Kennung, STEUERID, steuerid_ok),
        k!("idnr", "Patienten-/Kunden-/Fallnummer", "ID", Kennung, IDNUMMER),
        k!("birth", "Geburtsdatum", "GEBURTSDATUM", Person, BIRTH),
        k!("email", "E-Mail-Adresse", "E-MAIL", Kontakt, EMAIL),
        k!("name", "Name", "NAME", Person, TITLE_NAME),
        k!("name", "Name", "NAME", Person, LABEL_NAME),
        k!("name", "Name", "NAME", Person, FIRST_NAME),
        k!("street", "Anschrift", "ADRESSE", Kontakt, STREET),
        k!("plz", "PLZ und Ort", "ORT", Kontakt, PLZORT, plz_ok),
        k!("phone", "Telefonnummer", "TELEFON", Kontakt, PHONE, phone_ok),
        k!("ip", "IP-Adresse", "IP", Technik, IPV4, ip_ok),
        k!("health", "Gesundheitsangabe", "GESUNDHEIT", Gesundheit, HEALTH),
        k!("icd", "ICD-10-Code", "ICD", Gesundheit, ICD),
        k!("sensibel", "Besondere Kategorie (Religion, Herkunft …)", "SENSIBEL", Sensibel, SENSIBEL),
    ]
});

pub struct Finding {
    pub start: usize,
    pub end: usize,
    pub kind: usize,
}

pub fn find(text: &str) -> Vec<Finding> {
    let health_context = HEALTH.is_match(text);
    let mut out: Vec<Finding> = vec![];
    for (ki, k) in KINDS.iter().enumerate() {
        for c in k.re.captures_iter(text) {
            let m = c.get(1).unwrap_or_else(|| c.get(0).unwrap()); // Gruppe 1 = nur der Wert, z. B. ohne „Herr“
            let s = m.as_str();
            if !(k.check)(s) {
                continue;
            }
            // ICD-Codes ohne Punkt/Zusatz (z. B. „E11“) nur, wenn der Text ohnehin medizinisch ist.
            if k.id == "icd" && !health_context && !s.contains('.') && !s.ends_with(|c: char| c.is_ascii_alphabetic()) {
                continue;
            }
            if out.iter().any(|f| m.start() < f.end && f.start < m.end()) {
                continue; // höherrangiger Fund überdeckt das schon
            }
            out.push(Finding { start: m.start(), end: m.end(), kind: ki });
        }
    }
    out.sort_by_key(|f| f.start);
    out
}

fn digits(s: &str) -> Vec<u32> {
    s.chars().filter_map(|c| c.to_digit(10)).collect()
}

fn iban_ok(s: &str) -> bool {
    let s: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    if s.len() < 15 || s.len() > 34 {
        return false;
    }
    let r = format!("{}{}", &s[4..], &s[..4]);
    let mut m = 0u64;
    for c in r.chars() {
        let v = c.to_digit(36).unwrap_or(0) as u64;
        m = if v >= 10 { (m * 100 + v) % 97 } else { (m * 10 + v) % 97 };
    }
    m == 1
}

fn luhn_ok(s: &str) -> bool {
    let d = digits(s);
    if d.len() < 13 || d.len() > 19 {
        return false;
    }
    let sum: u32 = d.iter().rev().enumerate().map(|(i, &x)| if i % 2 == 1 { let y = x * 2; if y > 9 { y - 9 } else { y } } else { x }).sum();
    sum % 10 == 0
}

/// KVNR: Buchstabe (A=01 … Z=26) + 8 Ziffern, Gewichte 1-2-1-2, Quersummen, mod 10 = Prüfziffer.
fn kvnr_ok(s: &str) -> bool {
    let b = s.as_bytes();
    let l = (b[0] - b'A' + 1) as u32;
    let mut d = vec![l / 10, l % 10];
    d.extend(digits(&s[1..9]));
    let sum: u32 = d.iter().enumerate().map(|(i, &x)| { let p = x * if i % 2 == 0 { 1 } else { 2 }; p / 10 + p % 10 }).sum();
    sum % 10 == (b[9] - b'0') as u32
}

/// Steuer-ID: ISO 7064 MOD 11,10; außerdem kommt genau eine Ziffer doppelt oder dreifach vor.
fn steuerid_ok(s: &str) -> bool {
    let d = digits(s);
    if d.len() != 11 {
        return false;
    }
    let mut counts = [0u8; 10];
    d[..10].iter().for_each(|&x| counts[x as usize] += 1);
    if counts.iter().filter(|&&c| c >= 2).count() != 1 || counts.iter().any(|&c| c > 3) {
        return false;
    }
    let mut p = 10;
    for &x in &d[..10] {
        let mut sum = (x + p) % 10;
        if sum == 0 {
            sum = 10;
        }
        p = (sum * 2) % 11;
    }
    let check = if 11 - p == 10 { 0 } else { 11 - p };
    check == d[10]
}

fn phone_ok(s: &str) -> bool {
    let n = digits(s).len();
    (7..=15).contains(&n) && !s.contains('.')
}

fn plz_ok(s: &str) -> bool {
    let word = s[5..].trim().split([' ', '-']).next().unwrap_or("");
    !["Euro", "Dollar", "Stück", "Dateien", "Mitarbeiter", "Einwohner", "Patienten", "Kunden", "Zeichen", "Meter", "Kilometer", "Byte", "Zeilen", "Seiten"].contains(&word)
}

fn ip_ok(s: &str) -> bool {
    let p: Vec<u32> = s.split('.').filter_map(|x| x.parse().ok()).collect();
    p.len() == 4 && p.iter().all(|&x| x <= 255) && p != [0, 0, 0, 0] && !(p[0] == 127)
}

// ---------- Bewertung ----------

#[derive(Serialize)]
pub struct Category {
    id: &'static str,
    label: &'static str,
    group: Group,
    count: usize,
}

#[derive(Serialize)]
pub struct Segment {
    t: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    k: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    group: Option<Group>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ph: Option<String>,
}

#[derive(Serialize)]
pub struct Report {
    pub level: &'static str, // gruen | gelb | rot
    pub score: u32,
    pub headline: &'static str,
    pub reasons: Vec<String>,
    pub categories: Vec<Category>,
    pub segments: Vec<Segment>,
    pub redacted: String,
    pub chars: usize,
}

pub fn analyze(text: &str) -> Report {
    let found = find(text);
    let mut counts: Vec<(usize, usize)> = vec![]; // (kind, count) in Reihenfolge des Auftretens
    let mut per_group: HashMap<Group, usize> = HashMap::new();
    let mut tags: HashMap<(&str, &str), String> = HashMap::new();
    let mut tag_n: HashMap<&str, usize> = HashMap::new();
    let mut segments = vec![];
    let mut redacted = String::with_capacity(text.len());
    let mut pos = 0;

    for f in &found {
        let k = &KINDS[f.kind];
        *per_group.entry(k.group).or_default() += 1;
        match counts.iter_mut().find(|(ki, _)| KINDS[*ki].label == k.label) {
            Some(c) => c.1 += 1,
            None => counts.push((f.kind, 1)),
        }
        let value = &text[f.start..f.end];
        // Gleicher Wert → gleicher Platzhalter, damit die KI Bezüge noch versteht.
        let ph = tags
            .entry((k.tag, value))
            .or_insert_with(|| {
                let n = tag_n.entry(k.tag).or_default();
                *n += 1;
                format!("[{} {}]", k.tag, n)
            })
            .clone();
        if f.start > pos {
            segments.push(Segment { t: text[pos..f.start].into(), k: None, label: None, group: None, ph: None });
            redacted.push_str(&text[pos..f.start]);
        }
        redacted.push_str(&ph);
        segments.push(Segment { t: value.into(), k: Some(k.id), label: Some(k.label), group: Some(k.group), ph: Some(ph) });
        pos = f.end;
    }
    if pos < text.len() {
        segments.push(Segment { t: text[pos..].into(), k: None, label: None, group: None, ph: None });
        redacted.push_str(&text[pos..]);
    }

    let g = |x: Group| per_group.get(&x).copied().unwrap_or(0);
    let personal = g(Group::Person) + g(Group::Kontakt) + g(Group::Kennung) + g(Group::Finanz);
    let mut reasons = vec![];
    let mut level = "gruen";
    let mut headline = "Keine personenbezogenen Daten gefunden";
    let mut red = |r: &str, h: &'static str, level: &mut &str, headline: &mut &'static str| {
        reasons.push(r.to_string());
        if *level != "rot" {
            *level = "rot";
            *headline = h;
        }
    };
    if g(Group::Geheim) > 0 {
        red("Zugangsdaten wie Passwörter oder Schlüssel gehören nie in eine KI – sofort entfernen und ggf. ändern.", "Zugangsdaten gefunden", &mut level, &mut headline);
    }
    if g(Group::Gesundheit) > 0 && personal > 0 {
        red("Gesundheitsdaten mit Personenbezug sind eine besondere Kategorie nach Art. 9 DSGVO und dürfen nicht in eine externe KI.", "Gesundheitsdaten mit Personenbezug", &mut level, &mut headline);
    }
    if g(Group::Sensibel) > 0 && personal > 0 {
        red("Angaben zu Religion, Herkunft, Vorstrafen o. Ä. mit Personenbezug sind besonders geschützt (Art. 9/10 DSGVO).", "Besonders geschützte Daten", &mut level, &mut headline);
    }
    if g(Group::Finanz) + g(Group::Kennung) > 0 {
        red("Eindeutige Kennungen (IBAN, Versicherten-, Steuer- oder Patientennummer) identifizieren eine Person direkt.", "Eindeutige Kennungen gefunden", &mut level, &mut headline);
    }
    if level != "rot" {
        if personal > 0 {
            level = "gelb";
            headline = "Personenbezogene Daten gefunden";
            reasons.push("Namen, Kontakt- oder Adressdaten: vor der Eingabe anonymisieren – die anonymisierte Fassung unten ist dafür gedacht.".into());
        } else if g(Group::Gesundheit) + g(Group::Sensibel) > 0 {
            level = "gelb";
            headline = "Sensible Themen ohne erkennbaren Namen";
            reasons.push("Medizinische oder sensible Inhalte, aber keine Person erkannt. Prüfen, ob sich aus dem Zusammenhang doch jemand erkennen lässt.".into());
        } else if g(Group::Technik) > 0 {
            level = "gelb";
            headline = "Interne Technikdaten gefunden";
            reasons.push("IP-Adressen verraten den Netzaufbau und können Personen zugeordnet werden.".into());
        }
    }
    let n = found.len() as u32;
    let score = match level {
        "rot" => 70 + (n * 2).min(30),
        "gelb" => 35 + (n * 3).min(30),
        _ => 0,
    };
    let categories = counts.into_iter().map(|(ki, count)| { let k = &KINDS[ki]; Category { id: k.id, label: k.label, group: k.group, count } }).collect();
    Report { level, score, headline, reasons, categories, segments, redacted, chars: text.chars().count() }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(t: &str) -> Vec<&'static str> {
        find(t).iter().map(|f| KINDS[f.kind].id).collect()
    }

    #[test]
    fn pruefziffern() {
        assert!(iban_ok("DE89 3704 0044 0532 0130 00"));
        assert!(!iban_ok("DE89 3704 0044 0532 0130 01"));
        assert!(luhn_ok("4111 1111 1111 1111"));
        assert!(!luhn_ok("4111 1111 1111 1112"));
        assert!(kvnr_ok("A123456780"));
        assert!(!kvnr_ok("A123456781"));
        assert!(steuerid_ok("86095742719"));
        assert!(!steuerid_ok("86095742718"));
    }

    #[test]
    fn findet_typische_daten() {
        let t = "Patient: Müller, Hans, geb. 03.04.1961, KVNR A123456780\nDiagnose: Diabetes mellitus E11.9\nFrau Dr. Schneider, Hauptstraße 12, 12345 Berlin\nTel. 030 1234567, mail hans@example.de, IBAN DE89 3704 0044 0532 0130 00\nAdmin-Passwort: Geheim123!";
        let k = kinds(t);
        for want in ["name", "birth", "kvnr", "health", "icd", "street", "plz", "phone", "email", "iban", "password"] {
            assert!(k.contains(&want), "{want} fehlt in {k:?}");
        }
        let r = analyze(t);
        assert_eq!(r.level, "rot");
        assert!(!r.redacted.contains("Müller") && !r.redacted.contains("Geheim123"));
        assert!(r.redacted.contains("Patient: [NAME 1]"));
    }

    #[test]
    fn harmloser_text_bleibt_gruen() {
        let r = analyze("Bitte formuliere eine freundliche Mail an das Team: Das Meeting am Montag verschiebt sich auf 14 Uhr. Version 1.2.3 ist live, 10000 Euro Budget.");
        assert_eq!(r.level, "gruen", "{:?}", r.categories.iter().map(|c| c.id).collect::<Vec<_>>());
    }

    #[test]
    fn gleicher_wert_gleicher_platzhalter() {
        let r = analyze("Herr Meier kam. Später rief Herr Meier an, dann Frau Schulz.");
        assert!(r.redacted.contains("Herr [NAME 1] kam. Später rief Herr [NAME 1] an, dann Frau [NAME 2]."), "{}", r.redacted);
    }
}
