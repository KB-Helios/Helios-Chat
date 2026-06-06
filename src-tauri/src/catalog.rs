use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CatalogModel {
    pub id: String,
    pub name: String,
    pub description: String,
    pub hf_repo: String,
    pub hf_file: String,
    pub download_url: String,
    pub size_bytes: u64,
    pub sha256: Option<String>,
    pub quantization: String,
    pub parameters: String,
    pub recommended: bool,
    pub minimum_vram_gb: u8,
    pub tags: Vec<String>,
}

pub fn load_builtin_catalog() -> anyhow::Result<Vec<CatalogModel>> {
    let raw = include_str!("../resources/models.catalog.json");
    Ok(serde_json::from_str(raw)?)
}

pub fn recommended_model(catalog: &[CatalogModel]) -> Option<&CatalogModel> {
    catalog.iter().find(|model| model.recommended).or_else(|| catalog.first())
}

pub fn catalog_by_id(catalog: &[CatalogModel]) -> BTreeMap<String, CatalogModel> {
    catalog
        .iter()
        .cloned()
        .map(|model| (model.id.clone(), model))
        .collect()
}
