//! Cloudflare API client for DNS record management

use std::net::{Ipv4Addr, Ipv6Addr};

use anyhow::{Result, anyhow};
use reqwest::Client;
use serde::{Deserialize, Serialize};

const API_BASE: &str = "https://api.cloudflare.com/client/v4";

/// Cloudflare API client
#[derive(Clone)]
pub struct CloudflareClient {
    client: Client,
    token: String,
}

impl CloudflareClient {
    pub fn new(token: String) -> Self {
        Self {
            client: Client::new(),
            token,
        }
    }

    /// Verify the API token is valid
    pub async fn verify_token(&self) -> Result<bool> {
        let resp: ApiResponse<TokenVerifyResult> = self
            .client
            .get(format!("{}/user/tokens/verify", API_BASE))
            .bearer_auth(&self.token)
            .send()
            .await?
            .json()
            .await?;

        Ok(resp.success && resp.result.map(|r| r.status == "active").unwrap_or(false))
    }

    /// List all zones (domains) accessible with this token
    pub async fn list_zones(&self) -> Result<Vec<Zone>> {
        let mut all_zones = Vec::new();
        let mut page = 1;

        loop {
            let resp: ApiResponse<Vec<Zone>> = self
                .client
                .get(format!("{}/zones", API_BASE))
                .bearer_auth(&self.token)
                .query(&[("page", page.to_string()), ("per_page", "50".to_string())])
                .send()
                .await?
                .json()
                .await?;

            if !resp.success {
                return Err(anyhow!(
                    "Failed to list zones: {:?}",
                    resp.errors
                        .first()
                        .map(|e| e.message.clone())
                        .unwrap_or_default()
                ));
            }

            let zones = resp.result.unwrap_or_default();
            let is_last_page = zones.is_empty()
                || resp
                    .result_info
                    .map(|info| page >= info.total_pages)
                    .unwrap_or(true);

            all_zones.extend(zones);

            if is_last_page {
                break;
            }
            page += 1;
        }

        Ok(all_zones)
    }

    /// List DNS records for a zone
    pub async fn list_dns_records(&self, zone_id: &str) -> Result<Vec<DnsRecord>> {
        let mut all_records = Vec::new();
        let mut page = 1;

        loop {
            let resp: ApiResponse<Vec<DnsRecord>> = self
                .client
                .get(format!("{}/zones/{}/dns_records", API_BASE, zone_id))
                .bearer_auth(&self.token)
                .query(&[("page", page.to_string()), ("per_page", "100".to_string())])
                .send()
                .await?
                .json()
                .await?;

            if !resp.success {
                return Err(anyhow!(
                    "Failed to list DNS records: {:?}",
                    resp.errors
                        .first()
                        .map(|e| e.message.clone())
                        .unwrap_or_default()
                ));
            }

            let records = resp.result.unwrap_or_default();
            let is_last_page = records.is_empty()
                || resp
                    .result_info
                    .map(|info| page >= info.total_pages)
                    .unwrap_or(true);

            all_records.extend(records);

            if is_last_page {
                break;
            }
            page += 1;
        }

        Ok(all_records)
    }

    /// Create a new DNS record
    pub async fn create_dns_record(
        &self,
        zone_id: &str,
        record: &CreateDnsRecord,
    ) -> Result<DnsRecord> {
        let resp: ApiResponse<DnsRecord> = self
            .client
            .post(format!("{}/zones/{}/dns_records", API_BASE, zone_id))
            .bearer_auth(&self.token)
            .json(record)
            .send()
            .await?
            .json()
            .await?;

        if !resp.success {
            return Err(anyhow!(
                "Failed to create DNS record: {:?}",
                resp.errors
                    .first()
                    .map(|e| e.message.clone())
                    .unwrap_or_default()
            ));
        }

        resp.result.ok_or_else(|| anyhow!("No result returned"))
    }

    /// Update an existing DNS record
    pub async fn update_dns_record(
        &self,
        zone_id: &str,
        record_id: &str,
        record: &UpdateDnsRecord,
    ) -> Result<DnsRecord> {
        let resp: ApiResponse<DnsRecord> = self
            .client
            .patch(format!(
                "{}/zones/{}/dns_records/{}",
                API_BASE, zone_id, record_id
            ))
            .bearer_auth(&self.token)
            .json(record)
            .send()
            .await?
            .json()
            .await?;

        if !resp.success {
            return Err(anyhow!(
                "Failed to update DNS record: {:?}",
                resp.errors
                    .first()
                    .map(|e| e.message.clone())
                    .unwrap_or_default()
            ));
        }

        resp.result.ok_or_else(|| anyhow!("No result returned"))
    }

    /// Delete a DNS record
    pub async fn delete_dns_record(&self, zone_id: &str, record_id: &str) -> Result<()> {
        let resp: ApiResponse<DeleteResult> = self
            .client
            .delete(format!(
                "{}/zones/{}/dns_records/{}",
                API_BASE, zone_id, record_id
            ))
            .bearer_auth(&self.token)
            .send()
            .await?
            .json()
            .await?;

        if !resp.success {
            return Err(anyhow!(
                "Failed to delete DNS record: {:?}",
                resp.errors
                    .first()
                    .map(|e| e.message.clone())
                    .unwrap_or_default()
            ));
        }

        Ok(())
    }
}

// API Response types

#[derive(Debug, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub result: Option<T>,
    #[serde(default)]
    pub errors: Vec<ApiError>,
    pub result_info: Option<ResultInfo>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ApiError {
    pub code: i32,
    pub message: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ResultInfo {
    pub page: u32,
    pub per_page: u32,
    pub count: u32,
    pub total_count: u32,
    pub total_pages: u32,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct TokenVerifyResult {
    pub id: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct DeleteResult {
    pub id: String,
}

// Zone types

#[derive(Debug, Clone, Deserialize)]
pub struct Zone {
    pub id: String,
    pub name: String,
    pub status: String,
    pub account: ZoneAccount,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ZoneAccount {
    pub id: String,
    pub name: String,
}

// DNS Record types

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DnsRecord {
    pub id: String,
    #[serde(rename = "type")]
    pub record_type: DnsRecordType,
    pub name: String,
    pub content: String,
    pub ttl: u32,
    #[serde(default)]
    pub proxied: bool,
    #[serde(default)]
    pub proxiable: bool,
    #[serde(default)]
    pub priority: Option<u16>,
    #[serde(default)]
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "UPPERCASE")]
#[allow(clippy::upper_case_acronyms)]
pub enum DnsRecordType {
    A,
    AAAA,
    CNAME,
    MX,
    TXT,
    NS,
    SRV,
    CAA,
    PTR,
    #[serde(other)]
    Other,
}

impl DnsRecordType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DnsRecordType::A => "A",
            DnsRecordType::AAAA => "AAAA",
            DnsRecordType::CNAME => "CNAME",
            DnsRecordType::MX => "MX",
            DnsRecordType::TXT => "TXT",
            DnsRecordType::NS => "NS",
            DnsRecordType::SRV => "SRV",
            DnsRecordType::CAA => "CAA",
            DnsRecordType::PTR => "PTR",
            DnsRecordType::Other => "Other",
        }
    }

    pub fn all() -> &'static [DnsRecordType] {
        &[
            DnsRecordType::A,
            DnsRecordType::AAAA,
            DnsRecordType::CNAME,
            DnsRecordType::MX,
            DnsRecordType::TXT,
            DnsRecordType::NS,
            DnsRecordType::SRV,
            DnsRecordType::CAA,
            DnsRecordType::PTR,
        ]
    }

    /// Check if this record type can be proxied through Cloudflare
    pub fn is_proxiable(&self) -> bool {
        matches!(
            self,
            DnsRecordType::A | DnsRecordType::AAAA | DnsRecordType::CNAME
        )
    }

    /// Check if this record type requires a priority field
    pub fn requires_priority(&self) -> bool {
        matches!(self, DnsRecordType::MX | DnsRecordType::SRV)
    }

    /// Validate content for this record type
    pub fn validate_content(&self, content: &str) -> Result<(), &'static str> {
        match self {
            DnsRecordType::A => {
                content
                    .parse::<Ipv4Addr>()
                    .map_err(|_| "Invalid IPv4 address")?;
                Ok(())
            }
            DnsRecordType::AAAA => {
                content
                    .parse::<Ipv6Addr>()
                    .map_err(|_| "Invalid IPv6 address")?;
                Ok(())
            }
            DnsRecordType::MX | DnsRecordType::CNAME | DnsRecordType::NS | DnsRecordType::PTR => {
                if content.is_empty() {
                    return Err("Content cannot be empty");
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

impl std::fmt::Display for DnsRecordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateDnsRecord {
    #[serde(rename = "type")]
    pub record_type: DnsRecordType,
    pub name: String,
    pub content: String,
    pub ttl: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxied: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateDnsRecord {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub record_type: Option<DnsRecordType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxied: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}
