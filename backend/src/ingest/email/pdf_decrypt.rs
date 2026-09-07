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
    // The trailer's `/Encrypt` entry is the authoritative signal. Scanning the
    // raw bytes for the token is a good-enough heuristic and avoids a full parse.
    memchr_contains(bytes, b"/Encrypt")
}

/// Return a decrypted copy of `bytes` using `password`.
///
/// Works on RC4 and AES PDFs. A non-encrypted input is passed through
/// unchanged. An incorrect password (or a qpdf failure) is an `Err`.
pub fn decrypt_pdf(bytes: &[u8], password: &str) -> Result<Vec<u8>> {
    // `qpdf --password=PW --decrypt --stream-data=preserve - -`
    //   reads the PDF on stdin, writes a decrypted PDF to stdout.
    let mut child = std::process::Command::new("qpdf")
        .arg(format!("--password={password}"))
        .arg("--decrypt")
        .arg("--stream-data=preserve")
        .arg("--") // end of options
        .arg("-") // input: stdin
        .arg("-") // output: stdout
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to spawn qpdf (is it installed?)")?;

    child
        .stdin
        .take()
        .context("qpdf stdin unavailable")?
        .write_all(bytes)
        .context("failed to write PDF to qpdf")?;

    let out = child.wait_with_output().context("qpdf did not complete")?;

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

fn memchr_contains(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() || haystack.len() < needle.len() {
        return false;
    }
    haystack
        .windows(needle.len())
        .any(|w| w == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_encrypted_detects_token() {
        assert!(is_encrypted(b"%PDF-1.6\n... /Encrypt 12 0 R ..."));
        assert!(!is_encrypted(b"%PDF-1.4\nplain document"));
    }
}
