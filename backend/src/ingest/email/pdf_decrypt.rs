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
pub fn is_encrypted(bytes: &[u8]) -> bool {
    bytes.windows(8).any(|w| w == b"/Encrypt")
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

    let stderr = String::from_utf8_lossy(&out.stderr);
    let first = stderr.lines().next().unwrap_or("qpdf failed").trim();
    Err(anyhow!("qpdf: {first}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_encrypted_detects_token() {
        assert!(is_encrypted(b"%PDF-1.6\n... /Encrypt 12 0 R ..."));
        assert!(!is_encrypted(b"%PDF-1.4\nplain document"));
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
