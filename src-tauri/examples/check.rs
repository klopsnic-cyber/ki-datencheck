// Prüft Dateien von der Kommandozeile: cargo run --example check -- datei.pdf …
#[path = "../src/detect.rs"]
mod detect;
#[path = "../src/extract.rs"]
mod extract;

fn main() {
    for p in std::env::args().skip(1) {
        match extract::text_of(std::path::Path::new(&p)) {
            Ok(t) => {
                let r = detect::analyze(&t);
                println!("{p}: {} – {} ({} Funde)", r.level, r.headline, r.categories.len());
            }
            Err(e) => println!("{p}: FEHLER {e}"),
        }
    }
}
