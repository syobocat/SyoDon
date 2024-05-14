use base64::{prelude::BASE64_STANDARD, Engine};
use chrono::{DateTime, Utc};
use reqwest::header::HeaderMap;
use rsa::{
    pkcs1v15::SigningKey,
    sha2::{Digest, Sha256},
    signature::{SignatureEncoding, SignerMut},
};
use url::Url;

use super::Method;

pub fn create_header(
    method: Method,
    body: &serde_json::Value,
    date: DateTime<Utc>,
    dest: &Url,
) -> HeaderMap {
    let config = crate::CONFIG.get().unwrap();
    let url = &config.server.url;
    let host = url.host().unwrap();

    let dest_host = dest.host().unwrap();
    let dest_path = dest.path();

    let date_rfc7231 = date.to_rfc2822().replace("+0000", "GMT");

    let privkey = crate::PRIVKEY.get().unwrap().clone();
    let mut signingkey = SigningKey::<Sha256>::new(privkey);

    let mut header = HeaderMap::new();

    match method {
        Method::Get => {
            let signature_string =
                format!("(request-target): get {dest_path}\nhost: {host}\ndate: {date_rfc7231}");
            let hashed_signature = signingkey.sign(&signature_string.into_bytes());
            let signature_base64 = BASE64_STANDARD.encode(hashed_signature.to_bytes());
            header.insert(
                "Signature",
                format!(
                    "keyId=\"{url}actor#main-key\",headers=\"(request-target) host date\",signature=\"{signature_base64}\""
                ).parse().unwrap(),
            );
        }
        Method::Post => {
            let digest = Sha256::digest(serde_json::to_vec(&body).unwrap());
            let digest_base64 = BASE64_STANDARD.encode(digest);

            let signature_string = format!(
                "(request-target): post {dest_path}\nhost: {host}\ndate: {date_rfc7231}\ndigest: sha-256={digest_base64}"
            );
            let hashed_signature = signingkey.sign(&signature_string.into_bytes());
            let signature_base64 = BASE64_STANDARD.encode(hashed_signature.to_bytes());
            header.insert(
                "Digest",
                format!("sha-256={digest_base64}").parse().unwrap(),
            );
            header.insert(
                "Signature",
                format!(
                    "keyId=\"{url}actor#main-key\",algorithm=\"rsa-sha256\",headers=\"(request-target) host date digest\",signature=\"{signature_base64}\""
                ).parse().unwrap(),
            );
            header.insert(
                "Content-Type",
                "application/activity+json; charset=utf-8".parse().unwrap(),
            );
        }
    }

    header.insert("Host", dest_host.to_string().parse().unwrap());
    header.insert("Date", date_rfc7231.parse().unwrap());

    header
}
