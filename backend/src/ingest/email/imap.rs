//! IMAP access behind a trait so the sync engine can be tested with a fake
//! mailbox.

use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use futures::StreamExt;
use tokio::net::TcpStream;
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};
use tokio_rustls::TlsConnector;

/// One fetched message: its UID and the raw RFC822 bytes.
pub struct FetchedMessage {
    pub uid: u32,
    pub raw: Vec<u8>,
}

/// Server-side search terms. Attachment presence is filtered client-side.
#[derive(Debug, Default, Clone)]
pub struct SearchCriteria {
    pub from_any: Vec<String>,
    pub subject_any: Vec<String>,
}

impl SearchCriteria {
    /// Build the IMAP SEARCH query string for `UID <lo>:* <criteria>`.
    pub fn to_query(&self, since_uid: Option<u32>) -> String {
        let mut parts: Vec<String> = Vec::new();
        match since_uid {
            Some(u) => parts.push(format!("UID {}:*", u.saturating_add(1))),
            None => parts.push("ALL".to_string()),
        }
        parts.push(or_terms("FROM", &self.from_any));
        parts.push(or_terms("SUBJECT", &self.subject_any));
        parts
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// `OR FROM "a" OR FROM "b" FROM "c"` — IMAP's binary OR, left-folded.
fn or_terms(key: &str, values: &[String]) -> String {
    let quoted: Vec<String> = values
        .iter()
        .filter(|v| !v.trim().is_empty())
        .map(|v| format!("{key} {}", quote(v)))
        .collect();
    match quoted.len() {
        0 => String::new(),
        1 => quoted.into_iter().next().unwrap(),
        _ => {
            let mut it = quoted.into_iter();
            let mut acc = it.next().unwrap();
            for term in it {
                acc = format!("OR {acc} {term}");
            }
            acc
        }
    }
}

fn quote(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

#[async_trait]
pub trait Mailbox: Send {
    /// UIDVALIDITY of the selected folder.
    async fn uid_validity(&mut self) -> Result<u64>;
    /// UIDs matching `criteria`, ascending, capped to `limit` (oldest first).
    async fn search(&mut self, since_uid: Option<u32>, criteria: &SearchCriteria, limit: usize)
        -> Result<Vec<u32>>;
    /// Raw RFC822 bytes for the given UIDs.
    async fn fetch(&mut self, uids: &[u32]) -> Result<Vec<FetchedMessage>>;
    async fn logout(&mut self) -> Result<()>;
}

type TlsStream = tokio_rustls::client::TlsStream<TcpStream>;

/// Live IMAP-over-TLS session against Gmail (or any IMAPS server).
pub struct RustlsImap {
    session: async_imap::Session<TlsStream>,
    uid_validity: Option<u32>,
}

impl RustlsImap {
    /// Connect, LOGIN, SELECT INBOX.
    pub async fn connect(server: &str, user: &str, app_password: &str) -> Result<Self> {
        let (host, port) = split_host_port(server);

        let mut roots = RootCertStore::empty();
        roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        // Pin the crypto provider explicitly: with both `ring` and (transitively)
        // other providers in the tree, rustls refuses to guess.
        let config = ClientConfig::builder_with_provider(std::sync::Arc::new(
            tokio_rustls::rustls::crypto::ring::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .context("rustls protocol versions")?
        .with_root_certificates(roots)
        .with_no_client_auth();
        let connector = TlsConnector::from(Arc::new(config));

        let tcp = TcpStream::connect((host.as_str(), port))
            .await
            .with_context(|| format!("connect {host}:{port}"))?;
        let dns = ServerName::try_from(host.clone()).context("invalid IMAP host")?;
        let tls = connector.connect(dns, tcp).await.context("TLS handshake")?;

        let client = async_imap::Client::new(tls);
        let mut session = client
            .login(user, app_password)
            .await
            .map_err(|(e, _)| anyhow!("IMAP login failed: {e}"))?;

        let mailbox = session.select("INBOX").await.context("SELECT INBOX")?;
        let uid_validity = mailbox.uid_validity;

        Ok(Self { session, uid_validity })
    }
}

#[async_trait]
impl Mailbox for RustlsImap {
    async fn uid_validity(&mut self) -> Result<u64> {
        self.uid_validity
            .map(u64::from)
            .ok_or_else(|| anyhow!("server did not report UIDVALIDITY"))
    }

    async fn search(
        &mut self,
        since_uid: Option<u32>,
        criteria: &SearchCriteria,
        limit: usize,
    ) -> Result<Vec<u32>> {
        let query = criteria.to_query(since_uid);
        let hits = self
            .session
            .uid_search(&query)
            .await
            .with_context(|| format!("UID SEARCH {query}"))?;
        let mut uids: Vec<u32> = hits.into_iter().collect();
        uids.sort_unstable();
        uids.truncate(limit);
        Ok(uids)
    }

    async fn fetch(&mut self, uids: &[u32]) -> Result<Vec<FetchedMessage>> {
        if uids.is_empty() {
            return Ok(Vec::new());
        }
        let set = uids
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let mut stream = self
            .session
            .uid_fetch(&set, "(UID RFC822)")
            .await
            .context("UID FETCH")?;

        let mut out = Vec::new();
        while let Some(item) = stream.next().await {
            let fetch = item.context("fetch item")?;
            let uid = match fetch.uid {
                Some(u) => u,
                None => continue,
            };
            if let Some(body) = fetch.body().or_else(|| fetch.text()) {
                out.push(FetchedMessage { uid, raw: body.to_vec() });
            }
        }
        Ok(out)
    }

    async fn logout(&mut self) -> Result<()> {
        self.session.logout().await.context("LOGOUT")?;
        Ok(())
    }
}

fn split_host_port(server: &str) -> (String, u16) {
    match server.rsplit_once(':') {
        Some((h, p)) => (h.to_string(), p.parse().unwrap_or(993)),
        None => (server.to_string(), 993),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_watermark_only() {
        let c = SearchCriteria::default();
        assert_eq!(c.to_query(Some(41)), "UID 42:*");
        assert_eq!(c.to_query(None), "ALL");
    }

    #[test]
    fn query_with_filters() {
        let c = SearchCriteria {
            from_any: vec!["a@bank.com".into(), "b@bank.com".into()],
            subject_any: vec!["statement".into()],
        };
        assert_eq!(
            c.to_query(Some(10)),
            r#"UID 11:* OR FROM "a@bank.com" FROM "b@bank.com" SUBJECT "statement""#
        );
    }

    #[test]
    fn host_port_split() {
        assert_eq!(split_host_port("imap.gmail.com:993"), ("imap.gmail.com".into(), 993));
        assert_eq!(split_host_port("imap.gmail.com"), ("imap.gmail.com".into(), 993));
    }
}
