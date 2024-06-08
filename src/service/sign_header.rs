use std::collections::HashMap;

use base64::{prelude::BASE64_STANDARD, Engine};
use chrono::Utc;
use reqwest::header::HeaderMap;
use rsa::{
    pkcs1v15::{Signature, SigningKey, VerifyingKey},
    sha2::{Digest, Sha256},
    signature::{SignatureEncoding, SignerMut, Verifier},
};
use url::Url;

use crate::structs::Method;

pub fn create_header(method: Method, body: &serde_json::Value, dest: &Url) -> HeaderMap {
    let config = crate::CONFIG.get().unwrap();
    let url = &config.server.url;
    let host = url.host().unwrap();

    let dest_host = dest.host().unwrap();
    let dest_path = dest.path();

    let date = Utc::now();
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

pub async fn verify_header(
    dest: &Url,
    method: Method,
    header: HeaderMap,
) -> Result<(), Box<dyn std::error::Error>> {
    let host = header.get("Host").ok_or("missing Host header")?.to_str()?;
    let date = header.get("Date").ok_or("missing Date header")?.to_str()?;

    let data = match method {
        Method::Get => {
            format!("(request-target): get {dest}\nhost: {host}\ndate: {date}")
        }
        Method::Post => {
            let dest_path = dest.path();
            let digest = header
                .get("Digest")
                .ok_or("missing Digest header")?
                .to_str()?;
            format!("(request-target): post {dest_path}\nhost: {host}\ndate: {date}\ndigest: sha-256={digest}")
        }
    };

    let signature_header_raw = header.get("Signature").ok_or("missing Signature header")?;
    let signature_header_str = signature_header_raw.to_str()?;
    let signature_header = signature_header_str
        .split(',')
        .filter_map(|kv| kv.split_once('='))
        .map(|(k, v)| (k, v.trim_matches('"')))
        .collect::<HashMap<&str, &str>>();
    let signature_base64 = signature_header
        .get("signature")
        .ok_or("missing signature field")?;
    let signature = BASE64_STANDARD.decode(signature_base64)?;

    let key_id = signature_header.get("keyId").ok_or("missing keyId field")?;
    let key_url = Url::parse(
        key_id
            .split_once('#')
            .get_or_insert((key_id, "#main-key"))
            .0,
    )?;
    let publickey = super::user::get_pubkey(key_url).await?;
    let verifyingkey = VerifyingKey::<Sha256>::new(publickey);

    verifyingkey
        .verify(data.as_bytes(), &Signature::try_from(signature.as_slice())?)
        .map_err(|_| "signature verification failed".into())
}
