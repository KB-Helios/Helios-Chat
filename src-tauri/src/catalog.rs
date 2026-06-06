use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CatalogModel {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(alias = "hf_repo")]
    pub hf_repo: String,
    #[serde(alias = "hf_file")]
    pub hf_file: String,
    #[serde(alias = "download_url")]
    pub download_url: String,
    #[serde(alias = "size_bytes")]
    pub size_bytes: u64,
    pub sha256: Option<String>,
    pub quantization: String,
    pub parameters: String,
    pub recommended: bool,
    #[serde(alias = "minimum_vram_gb")]
    pub minimum_vram_gb: u8,
    pub tags: Vec<String>,
}

pub fn load_builtin_catalog() -> anyhow::Result<Vec<CatalogModel>> {
    let raw = include_str!("../resources/models.catalog.json");
    Ok(serde_json::from_str(raw)?)
}

pub fn recommended_model(catalog: &[CatalogModel]) -> Option<&CatalogModel> {
    catalog
        .iter()
        .find(|model| model.recommended)
        .or_else(|| catalog.first())
}

pub fn catalog_by_id(catalog: &[CatalogModel]) -> BTreeMap<String, CatalogModel> {
    catalog
        .iter()
        .cloned()
        .map(|model| (model.id.clone(), model))
        .collect()
}
