// Text aus Dateien holen: Office (docx/xlsx/pptx/odt…) sind ZIPs mit XML, PDF über pdf-extract,
// alles andere wird als Text gelesen.
use regex::Regex;
use std::io::Read;
use std::path::Path;
use std::sync::LazyLock;

const MAX: u64 = 50 * 1024 * 1024;

pub fn text_of(path: &Path) -> Result<String, String> {
    let size = std::fs::metadata(path).map_err(|e| e.to_string())?.len();
    if size > MAX {
        return Err("Datei ist größer als 50 MB".into());
    }
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    match ext.as_str() {
        "pdf" => pdf_extract::extract_text(path).map_err(|e| format!("PDF nicht lesbar: {e}")),
        "docx" | "docm" | "dotx" => office(path, |n| n.starts_with("word/") && (n.contains("document") || n.contains("header") || n.contains("footer") || n.contains("footnotes") || n.contains("comments"))),
        "xlsx" | "xlsm" => office(path, |n| n == "xl/sharedStrings.xml" || n.starts_with("xl/worksheets/sheet") || n.starts_with("xl/comments")),
        "pptx" => office(path, |n| n.starts_with("ppt/slides/slide") || n.starts_with("ppt/notesSlides/")),
        "odt" | "ods" | "odp" => office(path, |n| n == "content.xml"),
        "doc" | "xls" | "ppt" => Err("Altes Office-Format – bitte als .docx/.xlsx/.pptx speichern".into()),
        _ => {
            let b = std::fs::read(path).map_err(|e| e.to_string())?;
            if b.iter().take(8000).filter(|&&c| c == 0).count() > 8 {
                return Err("Binärdatei – kein Text enthalten".into());
            }
            Ok(String::from_utf8(b.clone()).unwrap_or_else(|_| b.iter().map(|&c| c as char).collect())) // sonst Latin-1
        }
    }
}

fn office(path: &Path, want: impl Fn(&str) -> bool) -> Result<String, String> {
    let f = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut z = zip::ZipArchive::new(f).map_err(|_| "Datei ist kein gültiges Office-Dokument".to_string())?;
    let mut names: Vec<String> = z.file_names().flatten().map(String::from).filter(|n| want(n)).collect();
    names.sort();
    let mut out = String::new();
    for n in names {
        let mut xml = String::new();
        if z.by_name(&n).and_then(|mut e| e.read_to_string(&mut xml).map_err(Into::into)).is_ok() {
            out.push_str(&xml_text(&xml));
            out.push('\n');
        }
    }
    Ok(out)
}

static BREAK: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"</(?:w:p|a:p|text:p|text:h|si|row)>|<w:br/>|<w:tab/>").unwrap());
static CELL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"</(?:c|table:table-cell)>").unwrap());
static TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]*>").unwrap());

fn xml_text(xml: &str) -> String {
    let s = BREAK.replace_all(xml, "\n");
    let s = CELL.replace_all(&s, " ");
    let s = TAG.replace_all(&s, "");
    s.replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&apos;", "'").replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    #[test]
    fn xml_wird_text() {
        assert_eq!(super::xml_text("<w:p><w:r><w:t>Herr Meier &amp; Co</w:t></w:r></w:p><w:p>x</w:p>"), "Herr Meier & Co\nx\n");
    }
}
