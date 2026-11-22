use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct OutdatedPkg {
    pub manager: String,
    pub name: String,
    pub current: String,
    pub latest: String,
    pub kind: String,
    pub license: String,
    pub cves: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransitiveVuln {
    pub ecosystem: String,
    pub package: String,
    pub version: String,
    pub ids: Vec<String>,
}
