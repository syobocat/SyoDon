use std::collections::HashMap;

use actix_web::http::Uri;
use base64::{prelude::BASE64_STANDARD, Engine};
use chrono::{DateTime, Utc};
use rsa::{
    pkcs1v15::SigningKey,
    sha2::{Digest, Sha256},
    signature::{SignatureEncoding, SignerMut},
};

use super::Method;

pub fn create_header(
    method: Method,
    body: serde_json::Value,
    date: DateTime<Utc>,
    dest: Uri,
) -> HashMap<String, String> {
    let config = crate::CONFIG.get().unwrap();
    let url = &config.server.url;
    let host = url.host().unwrap();

    let dest_host = dest.host().unwrap_or_default();
    let dest_path = dest.path();

    let date_rfc7231 = date.to_rfc2822().replace("+0000", "GMT");

    let privkey = crate::PRIVKEY.get().unwrap().clone();
    let mut signingkey = SigningKey::<Sha256>::new(privkey);

    let mut header = HashMap::new();

    match method {
        Method::Get => {
            let signature_string =
                format!("(request-target): get {dest_path}\nhost: {host}\ndate: {date_rfc7231}");
            let hashed_signature = signingkey.sign(&signature_string.into_bytes());
            let signature_base64 = BASE64_STANDARD.encode(hashed_signature.to_bytes());
            header.insert(
                "Signature".to_owned(),
                format!(
                    "keyId=\"{url}actor#main-key\",headers=\"(request-target) host date\",signature=\"{signature_base64}\""
                ),
            );
        }
        Method::Post => {
            let digest = Sha256::digest(&serde_json::to_vec(&body).unwrap());
            let digest_base64 = BASE64_STANDARD.encode(digest);

            let signature_string = format!(
                "(request-target): post {dest_path}\nhost: {host}\ndate: {date_rfc7231}\ndigest: sha-256={digest_base64}"
            );
            let hashed_signature = signingkey.sign(&signature_string.into_bytes());
            let signature_base64 = BASE64_STANDARD.encode(hashed_signature.to_bytes());
            header.insert("Digest".to_owned(), format!("sha-256={digest_base64}"));
            header.insert(
                "Signature".to_owned(),
                format!(
                    "keyId=\"{url}actor#main-key\",algorithm=\"rsa-sha256\",headers=\"(request-target) host date digest\",signature=\"{signature_base64}\""
                ),
            );
            header.insert(
                "Content-Type".to_owned(),
                "application/activity+json; charset=utf-8".to_owned(),
            );
        }
    }

    header.insert("Host".to_owned(), dest_host.to_owned());
    header.insert("Date".to_owned(), date_rfc7231);

    header
}
