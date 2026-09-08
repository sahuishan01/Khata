//! Decrypt password-protected statement PDFs via `qpdf`.
//!
//! `pdf-extract` (and lopdf) can only handle unencrypted or legacy RC4 PDFs;
//! virtually every modern bank statement uses AES. `qpdf` handles RC4, AES-128
//! and AES-256, so the worker and the manual upload handler both pipe suspect
//! PDFs through it before parsing.

use std::io::Write;
use std::process::Stdio;

use anyhow::{anyhow, Context, Result};

/// Cheap check: does the raw PDF declare an `/Encrypt` dictionary?
///
/// The bare `/Encrypt` token can appear inside content streams, names or
/// annotations, so a plain substring scan gives false positives that then get
/// force-fed to `qpdf`. A real encryption dict is `/Encrypt N G R` (indirect
/// reference, the common case) or `/Encrypt <<…>>` (inline) in the trailer, so
/// require one of those shapes.
pub fn is_encrypted(bytes: &[u8]) -> bool {
    const NEEDLE: &[u8] = b"/Encrypt";
    let is_ws = |b: u8| matches!(b, b' ' | b'\t' | b'\r' | b'\n' | b'\x0c' | b'\0');
    bytes.windows(NEEDLE.len()).enumerate().any(|(i, w)| {
        if w != NEEDLE {
            return false;
        }
        let mut rest = &bytes[i + NEEDLE.len()..];
        // Must be a delimiter right after the token, not e.g. "/Encryptable".
        match rest.first() {
            Some(&b) if is_ws(b) || b == b'<' => {}
            _ => return false,
        }
        while let Some(&b) = rest.first() {
            if is_ws(b) {
                rest = &rest[1..];
            } else {
                break;
            }
        }
        if rest.starts_with(b"<<") {
            return true;
        }
        // Indirect reference: <digits> <ws>+ <digits> <ws>+ R
        fn skip_digits(s: &[u8]) -> Option<&[u8]> {
            let n = s.iter().take_while(|b| b.is_ascii_digit()).count();
            (n > 0).then(|| &s[n..])
        }
        fn skip_ws1(s: &[u8]) -> Option<&[u8]> {
            let n = s
                .iter()
                .take_while(|b| matches!(b, b' ' | b'\t' | b'\r' | b'\n' | b'\x0c' | b'\0'))
                .count();
            (n > 0).then(|| &s[n..])
        }
        fn is_indirect_ref(s: &[u8]) -> Option<()> {
            let s = skip_digits(s)?;
            let s = skip_ws1(s)?;
            let s = skip_digits(s)?;
            let s = skip_ws1(s)?;
            s.starts_with(b"R").then_some(())
        }
        is_indirect_ref(rest).is_some()
    })
}

/// Return a decrypted copy of `bytes` using `password`.
///
/// Works on RC4 and AES PDFs. A non-encrypted input is passed through
/// unchanged. An incorrect password (or a qpdf failure) is an `Err`.
pub fn decrypt_pdf(bytes: &[u8], password: &str) -> Result<Vec<u8>> {
    // qpdf 10.x requires a real input file; the output may be `-` (stdout).
    // NamedTempFile creates the file with O_EXCL + mode 0600 and an
    // unpredictable name, so a pre-planted symlink in /tmp can't redirect the
    // write. It's removed on drop.
    let mut in_file = tempfile::Builder::new()
        .prefix("khata-pdf-")
        .suffix(".pdf")
        .tempfile()
        .context("create temp file for qpdf")?;
    in_file.write_all(bytes).context("stage PDF for qpdf")?;
    in_file.flush().ok();

    let out = std::process::Command::new("qpdf")
        .arg(format!("--password={password}"))
        .arg("--decrypt")
        .arg("--stream-data=preserve")
        .arg(in_file.path())
        .arg("-")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("failed to run qpdf (is it installed?)")?;

    // qpdf exit codes: 0 = ok, 3 = ok with warnings, 2 = errors.
    if out.status.success() || out.status.code() == Some(3) {
        if out.stdout.is_empty() {
            return Err(anyhow!("qpdf produced no output"));
        }
        return Ok(out.stdout);
    }

    // qpdf emits recoverable `WARNING:` lines first and the fatal error last,
    // so the first line ("reported number of objects…") is usually noise. Pick
    // the last meaningful line: skip blank lines and qpdf's generic summary.
    let stderr = String::from_utf8_lossy(&out.stderr);
    let msg = stderr
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("qpdf: operation"))
        .filter(|l| !l.starts_with("WARNING:"))
        .last()
        .or_else(|| {
            stderr
                .lines()
                .map(str::trim)
                .rev()
                .find(|l| !l.is_empty() && !l.starts_with("qpdf: operation"))
        })
        .unwrap_or("qpdf failed");
    Err(anyhow!("qpdf: {msg}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_encrypted_detects_token() {
        assert!(is_encrypted(b"%PDF-1.6\n... /Encrypt 12 0 R ..."));
        assert!(is_encrypted(b"trailer<</Size 9/Root 1 0 R/Encrypt 8 0 R>>"));
        assert!(is_encrypted(b"<</Filter/Standard/Encrypt<</V 4>>>>"));
        assert!(!is_encrypted(b"%PDF-1.4\nplain document"));
    }

    #[test]
    fn is_encrypted_ignores_false_positives() {
        // Bare token in a content stream, not followed by a ref or dict.
        assert!(!is_encrypted(b"BT (/Encrypt this text) Tj ET"));
        assert!(!is_encrypted(b"/Encryptable 3 0 R"));
        assert!(!is_encrypted(b"/Encrypt/Foo"));
    }

    #[test]
    #[ignore = "requires qpdf on PATH"]
    fn roundtrips_encrypted_pdf() {
        // Build a tiny encrypted PDF with qpdf itself, then decrypt it back.
        let plain = b"%PDF-1.4\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj\n\
                      2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj\n\
                      3 0 obj<</Type/Page/Parent 2 0 R/MediaBox[0 0 200 200]>>endobj\n\
                      trailer<</Size 4/Root 1 0 R>>\n%%EOF\n";
        let dir = std::env::temp_dir();
        let p = dir.join("kh-plain.pdf");
        let e = dir.join("kh-enc.pdf");
        std::fs::write(&p, plain).unwrap();
        // Normalise via qpdf first (our hand-rolled PDF has no real xref), then encrypt.
        let norm = dir.join("kh-norm.pdf");
        let _ = std::process::Command::new("qpdf").arg(&p).arg(&norm).status();
        let ok = std::process::Command::new("qpdf")
            .args(["--encrypt", "pw", "pw", "256", "--"])
            .arg(&norm)
            .arg(&e)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if !ok {
            eprintln!("skip: qpdf could not build the fixture");
            return;
        }
        let enc = std::fs::read(&e).unwrap();
        assert!(is_encrypted(&enc));
        let dec = decrypt_pdf(&enc, "pw").unwrap();
        assert!(dec.starts_with(b"%PDF"));
        assert!(!is_encrypted(&dec));
        let _ = std::fs::remove_file(&p);
        let _ = std::fs::remove_file(&e);
    }
}
