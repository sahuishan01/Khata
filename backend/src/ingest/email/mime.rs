//! Pull file attachments out of a raw RFC822 message.

use mail_parser::{MessageParser, MimeHeaders};

#[derive(Debug, Clone)]
pub struct Attachment {
    pub filename: String,
    pub bytes: Vec<u8>,
}

/// Return every attachment whose filename ends in a statement extension.
/// `max_bytes` attachments are still returned (the caller records the error);
/// callers cap total work elsewhere.
pub fn statement_attachments(raw_message: &[u8]) -> Vec<Attachment> {
    let Some(msg) = MessageParser::default().parse(raw_message) else {
        return Vec::new();
    };

    let mut out = Vec::new();
    for part in msg.attachments() {
        let name = part
            .attachment_name()
            .map(str::to_string)
            .unwrap_or_default();
        if name.is_empty() || !has_statement_ext(&name) {
            continue;
        }
        out.push(Attachment {
            filename: name,
            bytes: part.contents().to_vec(),
        });
    }
    out
}

/// Subject, From address, best-effort plain-text body and the mail's Date
/// (epoch seconds) — everything `txn_email::extract` needs from a message that
/// carries no statement attachment.
#[derive(Debug, Default, Clone)]
pub struct MessageMeta {
    pub subject: String,
    pub from: String,
    pub body: String,
    pub date_epoch: Option<i64>,
}

pub fn message_meta(raw_message: &[u8]) -> Option<MessageMeta> {
    let msg = MessageParser::default().parse(raw_message)?;

    let from = msg
        .from()
        .and_then(|a| a.first())
        .and_then(|addr| addr.address())
        .unwrap_or_default()
        .to_string();

    let body = msg
        .body_text(0)
        .map(|c| c.into_owned())
        .or_else(|| msg.body_html(0).map(|h| strip_html(&h)))
        .unwrap_or_default();

    Some(MessageMeta {
        subject: msg.subject().unwrap_or_default().to_string(),
        from,
        body,
        date_epoch: msg.date().map(|d| d.to_timestamp()),
    })
}

/// Crude tag strip — enough to run the alert-email regexes over an HTML-only body.
fn strip_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_tag = false;
    for ch in s.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out.replace("&nbsp;", " ").replace("&amp;", "&")
}

fn has_statement_ext(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    [".pdf", ".csv", ".xls", ".xlsx"]
        .iter()
        .any(|e| lower.ends_with(e))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &[u8] = b"\
From: bank@example.com\r\n\
To: me@example.com\r\n\
Subject: Statement\r\n\
MIME-Version: 1.0\r\n\
Content-Type: multipart/mixed; boundary=\"b\"\r\n\
\r\n\
--b\r\n\
Content-Type: text/plain\r\n\
\r\n\
Your statement is attached.\r\n\
--b\r\n\
Content-Type: application/pdf; name=\"stmt.pdf\"\r\n\
Content-Disposition: attachment; filename=\"stmt.pdf\"\r\n\
Content-Transfer-Encoding: base64\r\n\
\r\n\
JVBERi0xLjQK\r\n\
--b\r\n\
Content-Type: image/png; name=\"logo.png\"\r\n\
Content-Disposition: attachment; filename=\"logo.png\"\r\n\
Content-Transfer-Encoding: base64\r\n\
\r\n\
iVBORw0KGgo=\r\n\
--b--\r\n";

    #[test]
    fn extracts_only_statement_attachments() {
        let atts = statement_attachments(SAMPLE);
        assert_eq!(atts.len(), 1);
        assert_eq!(atts[0].filename, "stmt.pdf");
        assert_eq!(&atts[0].bytes[..5], b"%PDF-");
    }

    #[test]
    fn handles_garbage() {
        assert!(statement_attachments(b"not a message").is_empty());
    }
}
