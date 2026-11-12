//! Dex's gRPC API for local accounts (ORG-10, ADR-0060): list, create, reset and delete
//! accounts, and revoke their renewal tokens; and for connectors to upstream providers
//! (ORG-11, ADR-0063). Dex authenticates only by TLS client certificates;
//! `generate_certificates` makes the installation's CA and both certificates.
//!
//! The messages are the few needed from Dex's `api/v2/api.proto` (Apache-2.0), written as
//! prost structs with the same field numbers, so the build needs no `protoc`.

use std::time::Duration;

use tonic::transport::{Certificate, Channel, ClientTlsConfig, Endpoint, Identity};

/// Where Dex's API is and the certificates to reach it (`ORDO_DEX_API`,
/// `ORDO_DEX_API_CA_FILE`, `ORDO_DEX_API_CERT_FILE`, `ORDO_DEX_API_KEY_FILE`).
#[derive(Clone)]
pub struct DexApiConfig {
    /// Address, e.g. `https://dex:5557`.
    pub url: String,
    /// The installation's CA (PEM), which signed Dex's server certificate.
    pub ca: String,
    /// ORDO's client certificate (PEM).
    pub cert: String,
    /// ORDO's client key (PEM).
    pub key: String,
}

impl std::fmt::Debug for DexApiConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DexApiConfig")
            .field("url", &self.url)
            .finish_non_exhaustive()
    }
}

// --- Messages of Dex's api.proto (package `api`) ---

/// A local account as Dex keeps it.
#[derive(Clone, PartialEq, prost::Message)]
pub struct Password {
    /// Login name.
    #[prost(string, tag = "1")]
    pub email: String,
    /// bcrypt hash of the password.
    #[prost(bytes = "vec", tag = "2")]
    pub hash: Vec<u8>,
    /// The person's name (`name` claim).
    #[prost(string, tag = "3")]
    pub username: String,
    /// Dex's stable ID of the account (`sub` claim, encoded with the connector).
    #[prost(string, tag = "4")]
    pub user_id: String,
    /// Groups (unused by ORDO, ADR-0046; kept as they are).
    #[prost(string, repeated, tag = "5")]
    pub groups: Vec<String>,
}

#[derive(Clone, PartialEq, prost::Message)]
struct CreatePasswordReq {
    #[prost(message, optional, tag = "1")]
    password: Option<Password>,
}

#[derive(Clone, PartialEq, prost::Message)]
struct CreatePasswordResp {
    #[prost(bool, tag = "1")]
    already_exists: bool,
}

#[derive(Clone, PartialEq, prost::Message)]
struct UpdatePasswordReq {
    #[prost(string, tag = "1")]
    email: String,
    #[prost(bytes = "vec", tag = "2")]
    new_hash: Vec<u8>,
    #[prost(string, tag = "3")]
    new_username: String,
    #[prost(string, repeated, tag = "4")]
    new_groups: Vec<String>,
}

#[derive(Clone, PartialEq, prost::Message)]
struct UpdatePasswordResp {
    #[prost(bool, tag = "1")]
    not_found: bool,
}

#[derive(Clone, PartialEq, prost::Message)]
struct DeletePasswordReq {
    #[prost(string, tag = "1")]
    email: String,
}

#[derive(Clone, PartialEq, prost::Message)]
struct DeletePasswordResp {
    #[prost(bool, tag = "1")]
    not_found: bool,
}

#[derive(Clone, PartialEq, prost::Message)]
struct ListPasswordReq {}

#[derive(Clone, PartialEq, prost::Message)]
struct ListPasswordResp {
    #[prost(message, repeated, tag = "1")]
    passwords: Vec<Password>,
}

/// How Dex builds the `sub` claim: its account ID and the connector, as protobuf in
/// base64url without padding (`internal.IDTokenSubject`).
#[derive(Clone, PartialEq, prost::Message)]
struct IdTokenSubject {
    #[prost(string, tag = "1")]
    user_id: String,
    #[prost(string, tag = "2")]
    conn_id: String,
}

/// The `sub` claim Dex gives a local account (connector `local`): ORDO stores it with the
/// person's login, so it finds the person of a deleted account.
pub fn local_subject(user_id: &str) -> String {
    use base64::Engine as _;
    use prost::Message as _;
    let bytes = IdTokenSubject {
        user_id: user_id.to_owned(),
        conn_id: "local".to_owned(),
    }
    .encode_to_vec();
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

#[derive(Clone, PartialEq, prost::Message)]
struct RevokeRefreshReq {
    /// The `sub` claim, see [`local_subject`].
    #[prost(string, tag = "1")]
    user_id: String,
    #[prost(string, tag = "2")]
    client_id: String,
}

#[derive(Clone, PartialEq, prost::Message)]
struct RevokeRefreshResp {
    #[prost(bool, tag = "1")]
    not_found: bool,
}

/// A connector to an upstream identity provider as Dex keeps it.
#[derive(Clone, PartialEq, prost::Message)]
pub struct Connector {
    /// ID, e.g. `organisation`; part of every `sub` claim through it.
    #[prost(string, tag = "1")]
    pub id: String,
    /// Kind, e.g. `oidc`, `ldap`, `local`.
    #[prost(string, tag = "2")]
    pub r#type: String,
    /// The button text on Dex's login page.
    #[prost(string, tag = "3")]
    pub name: String,
    /// The kind's configuration as JSON (for `oidc` with the client secret).
    #[prost(bytes = "vec", tag = "4")]
    pub config: Vec<u8>,
    /// Allowed grant types; empty: all.
    #[prost(string, repeated, tag = "5")]
    pub grant_types: Vec<String>,
}

#[derive(Clone, PartialEq, prost::Message)]
struct CreateConnectorReq {
    #[prost(message, optional, tag = "1")]
    connector: Option<Connector>,
}

#[derive(Clone, PartialEq, prost::Message)]
struct CreateConnectorResp {
    #[prost(bool, tag = "1")]
    already_exists: bool,
}

#[derive(Clone, PartialEq, prost::Message)]
struct UpdateConnectorReq {
    #[prost(string, tag = "1")]
    id: String,
    #[prost(string, tag = "2")]
    new_type: String,
    #[prost(string, tag = "3")]
    new_name: String,
    #[prost(bytes = "vec", tag = "4")]
    new_config: Vec<u8>,
}

#[derive(Clone, PartialEq, prost::Message)]
struct UpdateConnectorResp {
    #[prost(bool, tag = "1")]
    not_found: bool,
}

#[derive(Clone, PartialEq, prost::Message)]
struct DeleteConnectorReq {
    #[prost(string, tag = "1")]
    id: String,
}

#[derive(Clone, PartialEq, prost::Message)]
struct DeleteConnectorResp {
    #[prost(bool, tag = "1")]
    not_found: bool,
}

#[derive(Clone, PartialEq, prost::Message)]
struct ListConnectorReq {}

#[derive(Clone, PartialEq, prost::Message)]
struct ListConnectorResp {
    #[prost(message, repeated, tag = "1")]
    connectors: Vec<Connector>,
}

/// What went wrong with a call.
#[derive(Debug, thiserror::Error)]
pub enum DexApiError {
    /// The configuration or a certificate is unusable.
    #[error("Dex API configuration: {0}")]
    Config(String),
    /// Dex is not reachable.
    #[error("Dex API not reachable: {0}")]
    Unreachable(String),
    /// Dex refused the call, e.g. to change an account from its configuration.
    #[error("Dex API refused: {0}")]
    Refused(String),
}

impl From<tonic::Status> for DexApiError {
    fn from(status: tonic::Status) -> Self {
        match status.code() {
            tonic::Code::Unavailable | tonic::Code::DeadlineExceeded => {
                Self::Unreachable(status.message().to_owned())
            }
            // Dex answers refusals, e.g. for accounts from its configuration, as `Unknown`.
            _ => Self::Refused(status.message().to_owned()),
        }
    }
}

/// The outcome of a change Dex answers with a flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Done.
    Done,
    /// Creating: the account exists already.
    Exists,
    /// Changing or deleting: no such account.
    NotFound,
}

// [impl->dsn~local-accounts~1]
/// A connection to Dex's API (opened on first use, reopened as needed).
#[derive(Clone)]
pub struct DexApi {
    channel: Channel,
}

impl DexApi {
    /// Prepares the connection; nothing is sent yet.
    pub fn new(config: &DexApiConfig) -> Result<Self, DexApiError> {
        let tls = ClientTlsConfig::new()
            .ca_certificate(Certificate::from_pem(&config.ca))
            .identity(Identity::from_pem(&config.cert, &config.key));
        let channel = Endpoint::from_shared(config.url.clone())
            .map_err(|e| DexApiError::Config(e.to_string()))?
            .tls_config(tls)
            .map_err(|e| DexApiError::Config(e.to_string()))?
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(15))
            .connect_lazy();
        Ok(Self { channel })
    }

    async fn call<Req, Resp>(&self, method: &'static str, request: Req) -> Result<Resp, DexApiError>
    where
        Req: prost::Message + Send + Sync + 'static,
        Resp: prost::Message + Default + Send + Sync + 'static,
    {
        let mut grpc = tonic::client::Grpc::new(self.channel.clone());
        grpc.ready()
            .await
            .map_err(|e| DexApiError::Unreachable(e.to_string()))?;
        let path = tonic::codegen::http::uri::PathAndQuery::from_static(method);
        let codec = tonic_prost::ProstCodec::<Req, Resp>::default();
        Ok(grpc
            .unary(tonic::Request::new(request), path, codec)
            .await?
            .into_inner())
    }

    /// All local accounts, sorted by e-mail address.
    pub async fn list(&self) -> Result<Vec<Password>, DexApiError> {
        let resp: ListPasswordResp = self
            .call("/api.Dex/ListPasswords", ListPasswordReq {})
            .await?;
        let mut list = resp.passwords;
        list.sort_by(|a, b| a.email.cmp(&b.email));
        Ok(list)
    }

    /// Creates an account with a bcrypt `hash` and a new random account ID.
    pub async fn create(
        &self,
        email: &str,
        name: &str,
        hash: Vec<u8>,
    ) -> Result<Outcome, DexApiError> {
        let mut id = [0u8; 16];
        getrandom::fill(&mut id).map_err(|e| DexApiError::Config(e.to_string()))?;
        let password = Password {
            email: email.to_owned(),
            hash,
            username: name.to_owned(),
            user_id: crate::config::to_hex(&id),
            groups: Vec::new(),
        };
        let resp: CreatePasswordResp = self
            .call(
                "/api.Dex/CreatePassword",
                CreatePasswordReq {
                    password: Some(password),
                },
            )
            .await?;
        Ok(if resp.already_exists {
            Outcome::Exists
        } else {
            Outcome::Done
        })
    }

    /// Sets a new password (bcrypt `hash`); name and groups stay.
    pub async fn set_password(
        &self,
        account: &Password,
        hash: Vec<u8>,
    ) -> Result<Outcome, DexApiError> {
        let resp: UpdatePasswordResp = self
            .call(
                "/api.Dex/UpdatePassword",
                UpdatePasswordReq {
                    email: account.email.clone(),
                    new_hash: hash,
                    new_username: account.username.clone(),
                    new_groups: account.groups.clone(),
                },
            )
            .await?;
        Ok(if resp.not_found {
            Outcome::NotFound
        } else {
            Outcome::Done
        })
    }

    /// Deletes an account and revokes its renewal tokens for `client_id` (ORDO).
    pub async fn delete(
        &self,
        account: &Password,
        client_id: &str,
    ) -> Result<Outcome, DexApiError> {
        let resp: DeletePasswordResp = self
            .call(
                "/api.Dex/DeletePassword",
                DeletePasswordReq {
                    email: account.email.clone(),
                },
            )
            .await?;
        if resp.not_found {
            return Ok(Outcome::NotFound);
        }
        let _: RevokeRefreshResp = self
            .call(
                "/api.Dex/RevokeRefresh",
                RevokeRefreshReq {
                    user_id: local_subject(&account.user_id),
                    client_id: client_id.to_owned(),
                },
            )
            .await?;
        Ok(Outcome::Done)
    }
}

// [impl->dsn~login-providers~1]
impl DexApi {
    /// All connectors, sorted by ID.
    pub async fn connectors(&self) -> Result<Vec<Connector>, DexApiError> {
        let resp: ListConnectorResp = self
            .call("/api.Dex/ListConnectors", ListConnectorReq {})
            .await?;
        let mut list = resp.connectors;
        list.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(list)
    }

    /// Creates a connector.
    pub async fn create_connector(&self, connector: Connector) -> Result<Outcome, DexApiError> {
        let resp: CreateConnectorResp = self
            .call(
                "/api.Dex/CreateConnector",
                CreateConnectorReq {
                    connector: Some(connector),
                },
            )
            .await?;
        Ok(if resp.already_exists {
            Outcome::Exists
        } else {
            Outcome::Done
        })
    }

    /// Replaces a connector's kind, name and configuration; its ID stays.
    pub async fn update_connector(&self, connector: Connector) -> Result<Outcome, DexApiError> {
        let resp: UpdateConnectorResp = self
            .call(
                "/api.Dex/UpdateConnector",
                UpdateConnectorReq {
                    id: connector.id,
                    new_type: connector.r#type,
                    new_name: connector.name,
                    new_config: connector.config,
                },
            )
            .await?;
        Ok(if resp.not_found {
            Outcome::NotFound
        } else {
            Outcome::Done
        })
    }

    /// Deletes a connector.
    pub async fn delete_connector(&self, id: &str) -> Result<Outcome, DexApiError> {
        let resp: DeleteConnectorResp = self
            .call(
                "/api.Dex/DeleteConnector",
                DeleteConnectorReq { id: id.to_owned() },
            )
            .await?;
        Ok(if resp.not_found {
            Outcome::NotFound
        } else {
            Outcome::Done
        })
    }
}

/// bcrypt hash of a password, as Dex stores it (cost 10, Dex's default).
pub fn hash_password(password: &str) -> Result<Vec<u8>, DexApiError> {
    bcrypt::hash(password, 10)
        .map(String::into_bytes)
        .map_err(|e| DexApiError::Config(e.to_string()))
}

/// The certificates for Dex's API, as PEM: the CA, Dex's server certificate and key, and
/// ORDO's client certificate and key.
pub struct Certificates {
    /// The installation's CA (its key is not kept: new certificates mean a new CA).
    pub ca: String,
    /// Dex's server certificate.
    pub dex_cert: String,
    /// Dex's key.
    pub dex_key: String,
    /// ORDO's client certificate.
    pub ordo_cert: String,
    /// ORDO's key.
    pub ordo_key: String,
}

/// Makes a CA and, signed by it, Dex's server certificate for `dex_hosts` and ORDO's
/// client certificate, valid for ten years.
pub fn generate_certificates(dex_hosts: &[String]) -> Result<Certificates, rcgen::Error> {
    use rcgen::{
        BasicConstraints, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa, Issuer,
        KeyPair, KeyUsagePurpose,
    };
    let now = time::OffsetDateTime::now_utc();
    let valid = |params: &mut CertificateParams| {
        params.not_before = now - time::Duration::days(1);
        params.not_after = now + time::Duration::days(3653);
    };

    let ca_key = KeyPair::generate()?;
    let mut ca = CertificateParams::new(Vec::<String>::new())?;
    ca.distinguished_name
        .push(DnType::CommonName, "ORDO Dex API CA");
    ca.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));
    ca.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
    valid(&mut ca);
    let ca_cert = ca.self_signed(&ca_key)?;
    let issuer = Issuer::new(ca, ca_key);

    let dex_key = KeyPair::generate()?;
    let mut dex = CertificateParams::new(dex_hosts.to_vec())?;
    dex.distinguished_name.push(DnType::CommonName, "dex");
    dex.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    valid(&mut dex);
    let dex_cert = dex.signed_by(&dex_key, &issuer)?;

    let ordo_key = KeyPair::generate()?;
    let mut ordo = CertificateParams::new(Vec::<String>::new())?;
    ordo.distinguished_name.push(DnType::CommonName, "ordo");
    ordo.extended_key_usages = vec![ExtendedKeyUsagePurpose::ClientAuth];
    valid(&mut ordo);
    let ordo_cert = ordo.signed_by(&ordo_key, &issuer)?;

    Ok(Certificates {
        ca: ca_cert.pem(),
        dex_cert: dex_cert.pem(),
        dex_key: dex_key.serialize_pem(),
        ordo_cert: ordo_cert.pem(),
        ordo_key: ordo_key.serialize_pem(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // [utest->dsn~local-accounts~1]
    #[test]
    fn passwords_are_hashed_for_dex_and_certificates_made() {
        let hash = hash_password("ein langes Passwort").unwrap();
        let hash = String::from_utf8(hash).unwrap();
        assert!(hash.starts_with("$2b$10$"), "bcrypt, Dex's cost");
        assert!(bcrypt::verify("ein langes Passwort", &hash).unwrap());
        let certs = generate_certificates(&["dex".to_owned(), "127.0.0.1".to_owned()]).unwrap();
        assert!(certs.ca.starts_with("-----BEGIN CERTIFICATE-----"));
        assert!(certs.ordo_key.contains("PRIVATE KEY"));
        // Usable by the client.
        let config = DexApiConfig {
            url: "https://127.0.0.1:5557".to_owned(),
            ca: certs.ca,
            cert: certs.ordo_cert,
            key: certs.ordo_key,
        };
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        runtime.block_on(async { assert!(DexApi::new(&config).is_ok()) });
    }

    #[test]
    fn messages_match_dex_field_numbers() {
        use prost::Message;
        // CreatePasswordReq { password: { email: "a" } }: field 1 (message) holding field 1 (string).
        let bytes = CreatePasswordReq {
            password: Some(Password {
                email: "a".to_owned(),
                ..Default::default()
            }),
        }
        .encode_to_vec();
        assert_eq!(bytes, [0x0a, 0x03, 0x0a, 0x01, b'a']);
        let resp = ListPasswordResp::decode(&[0x0a, 0x03, 0x22, 0x01, b'x'][..]).unwrap();
        assert_eq!(resp.passwords[0].user_id, "x", "user_id is field 4");
        // [utest->dsn~login-providers~1]
        // Connector { id: "o", type: "oidc" }: fields 1 and 2.
        let bytes = CreateConnectorReq {
            connector: Some(Connector {
                id: "o".to_owned(),
                r#type: "oidc".to_owned(),
                ..Default::default()
            }),
        }
        .encode_to_vec();
        assert_eq!(
            bytes,
            [
                0x0a, 0x09, 0x0a, 0x01, b'o', 0x12, 0x04, b'o', b'i', b'd', b'c'
            ]
        );
        let update = UpdateConnectorReq {
            id: "o".to_owned(),
            new_config: b"{}".to_vec(),
            ..Default::default()
        }
        .encode_to_vec();
        assert_eq!(
            update,
            [0x0a, 0x01, b'o', 0x22, 0x02, b'{', b'}'],
            "config is field 4"
        );
        // As Dex 2.46 issues it for a static test account (seen in a login).
        assert_eq!(
            local_subject("8f8a6a4e-2d4b-4b3a-9b1e-0a6a1d1e0001"),
            "CiQ4ZjhhNmE0ZS0yZDRiLTRiM2EtOWIxZS0wYTZhMWQxZTAwMDESBWxvY2Fs"
        );
    }
}
