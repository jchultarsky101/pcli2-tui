use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcliFolder {
    pub id: String,
    pub name: String,
    pub path: String,
    #[serde(rename = "assetsCount")]
    pub assets_count: u32,
    #[serde(rename = "foldersCount")]
    pub folders_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcliAsset {
    pub uuid: String,
    pub name: String,
    pub path: String,
    #[serde(rename = "file_type")]
    pub file_type: String,
    #[serde(rename = "file_size")]
    pub file_size: Option<u64>,
    #[serde(rename = "processing_status")]
    pub processing_status: String,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
    pub metadata: serde_json::Value,
    #[serde(rename = "is_assembly")]
    pub is_assembly: bool,
}

// Functions to interact with pcli2
pub fn list_folders() -> Result<Vec<PcliFolder>> {
    let output = Command::new("pcli2")
        .args(["folder", "list", "--format", "json"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("pcli2 folder list failed: {}", stderr));
    }

    let stdout = String::from_utf8(output.stdout)?;
    let folders: Vec<PcliFolder> = serde_json::from_str(&stdout)?;

    Ok(folders)
}

pub fn list_subfolders_of_folder(folder_path: &str) -> Result<Vec<PcliFolder>> {
    // Use folder list with --folder-path to get subfolders of a specific folder
    let output = Command::new("pcli2")
        .args([
            "folder",
            "list",
            "--folder-path",
            folder_path,
            "--format",
            "json",
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("pcli2 folder list failed: {}", stderr));
    }

    let stdout = String::from_utf8(output.stdout)?;
    let subfolders: Vec<PcliFolder> = serde_json::from_str(&stdout)?;

    Ok(subfolders)
}

pub fn list_assets_in_folder(folder_path: &str) -> Result<Vec<PcliAsset>> {
    let output = Command::new("pcli2")
        .args([
            "asset",
            "list",
            "--folder-path",
            folder_path,
            "--format",
            "json",
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("pcli2 asset list failed: {}", stderr));
    }

    let stdout = String::from_utf8(output.stdout)?;
    let assets: Vec<PcliAsset> = serde_json::from_str(&stdout)?;

    Ok(assets)
}

pub fn download_asset(asset_uuid: &str) -> Result<()> {
    let output = Command::new("pcli2")
        .args(["asset", "download", "--uuid", asset_uuid])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("pcli2 asset download failed: {}", stderr));
    }

    Ok(())
}

#[allow(dead_code)]
pub fn upload_asset_to_folder(file_path: &str, folder_uuid: &str) -> Result<()> {
    let output = Command::new("pcli2")
        .args([
            "asset",
            "create",
            "--file",
            file_path,
            "--folder",
            folder_uuid,
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("pcli2 asset upload failed: {}", stderr));
    }

    Ok(())
}

// Define structures for search results specifically
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SearchResultAsset {
    #[serde(rename = "id")]
    pub uuid: String,
    #[serde(rename = "path")]
    pub path: String,
    #[serde(rename = "type")]
    pub file_type: String,
    #[serde(rename = "tenantId")]
    pub tenant_id: Option<String>,
    #[serde(rename = "folderId")]
    pub folder_id: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
    #[serde(rename = "state")]
    pub state: Option<String>,
    #[serde(rename = "isAssembly")]
    pub is_assembly: Option<bool>,
    #[serde(rename = "file_size")]
    pub file_size: Option<u64>,
    #[serde(rename = "processing_status")]
    pub processing_status: Option<String>,
    #[serde(rename = "created_at")]
    pub created_at_legacy: Option<String>,
    #[serde(rename = "updated_at")]
    pub updated_at_legacy: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SearchResultMatch {
    asset: SearchResultAsset,
    #[serde(rename = "comparisonUrl")]
    comparison_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SearchResponse {
    #[serde(rename = "searchQuery")]
    search_query: String,
    matches: Vec<SearchResultMatch>,
}

pub fn search_assets(query: &str) -> Result<Vec<PcliAsset>> {
    // Use the exact working command with JSON format: pcli2 asset text-match --text <query> --format json
    let output = Command::new("pcli2")
        .args(["asset", "text-match", "--text", query, "--format", "json"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("pcli2 asset search failed: {}", stderr));
    }

    let stdout = String::from_utf8(output.stdout)?;

    // Parse the search results specifically using the search result structures
    match serde_json::from_str::<SearchResponse>(&stdout) {
        Ok(search_response) => {
            let assets: Vec<PcliAsset> = search_response.matches.into_iter()
                .map(|match_result| {
                    let search_asset = match_result.asset;
                    PcliAsset {
                        uuid: search_asset.uuid,
                        name: search_asset.path.split('/').last().unwrap_or(&search_asset.path).to_string(), // Extract filename from path
                        path: search_asset.path,
                        file_type: search_asset.file_type,
                        file_size: search_asset.file_size,
                        processing_status: search_asset.state.unwrap_or_else(|| "unknown".to_string()),
                        created_at: search_asset.created_at.unwrap_or_else(|| search_asset.created_at_legacy.unwrap_or_else(|| "unknown".to_string())),
                        updated_at: search_asset.updated_at.unwrap_or_else(|| search_asset.updated_at_legacy.unwrap_or_else(|| "unknown".to_string())),
                        metadata: search_asset.metadata.unwrap_or_else(|| serde_json::Value::Null),
                        is_assembly: search_asset.is_assembly.unwrap_or(false),
                    }
                })
                .collect();

            Ok(assets)
        }
        Err(_) => {
            // If parsing with dedicated structures fails, return an error with the raw output
            Err(anyhow::anyhow!(
                "Failed to parse search results as assets. Raw output: {}",
                stdout
            ))
        }
    }
}
