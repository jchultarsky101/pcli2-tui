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

use serde_json::Value;

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

    // Try to parse as an array first
    if let Ok(assets) = serde_json::from_str::<Vec<PcliAsset>>(&stdout) {
        return Ok(assets);
    }

    // If that fails, try to parse as a single object or wrapper object
    let json_value: Value = serde_json::from_str(&stdout)?;

    // Look for common patterns where the assets might be in a field
    if let Some(assets_array) = json_value.get("assets").and_then(|v| v.as_array()) {
        let assets: Result<Vec<PcliAsset>, _> = assets_array
            .iter()
            .map(|item| serde_json::from_value(item.clone()))
            .collect();
        if let Ok(assets) = assets {
            return Ok(assets);
        }
    } else if let Some(results_array) = json_value.get("results").and_then(|v| v.as_array()) {
        let assets: Result<Vec<PcliAsset>, _> = results_array
            .iter()
            .map(|item| serde_json::from_value(item.clone()))
            .collect();
        if let Ok(assets) = assets {
            return Ok(assets);
        }
    } else if let Some(data_array) = json_value.get("data").and_then(|v| v.as_array()) {
        let assets: Result<Vec<PcliAsset>, _> = data_array
            .iter()
            .map(|item| serde_json::from_value(item.clone()))
            .collect();
        if let Ok(assets) = assets {
            return Ok(assets);
        }
    }

    // If all parsing attempts fail, return an error
    Err(anyhow::anyhow!("Failed to parse search results as assets: {}", stdout))
}
