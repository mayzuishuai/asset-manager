//! JSON 文件存储实现

use super::StorageError;
use crate::asset::{Asset, AssetSummary, AssetTransaction, AssetType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;
use uuid::Uuid;

/// JSON 存储的数据结构
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JsonStore {
    /// 资产与负债
    pub assets: Vec<Asset>,
    /// 交易记录
    pub transactions: Vec<AssetTransaction>,
    /// 应用设置
    pub settings: HashMap<String, String>,
}

/// JSON 文件数据库
pub struct Database {
    path: Option<PathBuf>,
    store: JsonStore,
}

impl Database {
    /// 打开或创建 JSON 数据库文件
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let path = path.as_ref().to_path_buf();

        // 确保父目录存在
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let store = if path.exists() {
            let content = fs::read_to_string(&path)?;
            if content.trim().is_empty() {
                JsonStore::default()
            } else {
                serde_json::from_str(&content)?
            }
        } else {
            let store = JsonStore::default();
            let content = serde_json::to_string_pretty(&store)?;
            fs::write(&path, content)?;
            store
        };

        info!("JSON database opened: {:?}", path);

        Ok(Self {
            path: Some(path),
            store,
        })
    }

    /// 创建内存数据库（用于测试）
    pub fn open_in_memory() -> Result<Self, StorageError> {
        Ok(Self {
            path: None,
            store: JsonStore::default(),
        })
    }

    /// 将数据写入文件
    fn save(&self) -> Result<(), StorageError> {
        if let Some(ref path) = self.path {
            let content = serde_json::to_string_pretty(&self.store)?;
            fs::write(path, content)?;
        }
        Ok(())
    }

    // ============ 资产操作 ============

    /// 创建资产
    pub fn create_asset(&mut self, asset: &Asset) -> Result<(), StorageError> {
        self.store.assets.push(asset.clone());
        self.save()
    }

    /// 获取资产
    pub fn get_asset(&self, id: Uuid) -> Result<Option<Asset>, StorageError> {
        let asset = self.store.assets.iter().find(|a| a.id == id).cloned();
        Ok(asset)
    }

    /// 获取所有资产
    pub fn list_assets(&self) -> Result<Vec<Asset>, StorageError> {
        let mut assets = self.store.assets.clone();
        assets.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(assets)
    }

    /// 按类型获取资产
    pub fn list_assets_by_type(&self, asset_type: &AssetType) -> Result<Vec<Asset>, StorageError> {
        let mut assets: Vec<Asset> = self
            .store
            .assets
            .iter()
            .filter(|a| a.asset_type.as_str() == asset_type.as_str())
            .cloned()
            .collect();
        assets.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(assets)
    }

    /// 更新资产
    pub fn update_asset(&mut self, asset: &Asset) -> Result<(), StorageError> {
        let pos = self
            .store
            .assets
            .iter()
            .position(|a| a.id == asset.id)
            .ok_or_else(|| StorageError::NotFound(asset.id.to_string()))?;

        self.store.assets[pos] = asset.clone();
        self.save()
    }

    /// 删除资产
    pub fn delete_asset(&mut self, id: Uuid) -> Result<(), StorageError> {
        let pos = self
            .store
            .assets
            .iter()
            .position(|a| a.id == id)
            .ok_or_else(|| StorageError::NotFound(id.to_string()))?;

        self.store.assets.remove(pos);
        // 同时删除关联的交易记录
        self.store.transactions.retain(|t| t.asset_id != id);
        self.save()
    }

    /// 搜索资产
    pub fn search_assets(&self, query: &str) -> Result<Vec<Asset>, StorageError> {
        let query_lower = query.to_lowercase();
        let mut assets: Vec<Asset> = self
            .store
            .assets
            .iter()
            .filter(|a| {
                a.name.to_lowercase().contains(&query_lower)
                    || a.description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
                    || a.tags
                        .iter()
                        .any(|t| t.to_lowercase().contains(&query_lower))
            })
            .cloned()
            .collect();
        assets.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(assets)
    }

    // ============ 统计功能 ============

    /// 获取资产统计摘要
    pub fn get_summary(&self) -> Result<AssetSummary, StorageError> {
        let assets = self.list_assets()?;
        let mut summary = AssetSummary::default();
        summary.asset_count = assets.len();

        for asset in &assets {
            summary.total_value += asset.value;

            // 按类型统计
            let type_key = asset.asset_type.as_str().to_string();
            *summary.by_type.entry(type_key).or_insert(0.0) += asset.value;

            // 按货币统计
            let currency_key = format!("{:?}", asset.currency);
            *summary.by_currency.entry(currency_key).or_insert(0.0) += asset.value;
        }

        Ok(summary)
    }

    // ============ 交易记录 ============

    /// 记录交易
    pub fn add_transaction(&mut self, transaction: &AssetTransaction) -> Result<(), StorageError> {
        self.store.transactions.push(transaction.clone());
        self.save()
    }

    /// 获取资产的交易历史
    pub fn get_transactions(&self, asset_id: Uuid) -> Result<Vec<AssetTransaction>, StorageError> {
        let mut txns: Vec<AssetTransaction> = self
            .store
            .transactions
            .iter()
            .filter(|t| t.asset_id == asset_id)
            .cloned()
            .collect();
        txns.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(txns)
    }

    // ============ 设置 ============

    /// 保存设置
    pub fn set_setting(&mut self, key: &str, value: &str) -> Result<(), StorageError> {
        self.store
            .settings
            .insert(key.to_string(), value.to_string());
        self.save()
    }

    /// 获取设置
    pub fn get_setting(&self, key: &str) -> Result<Option<String>, StorageError> {
        Ok(self.store.settings.get(key).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::asset::{Asset, AssetType};

    #[test]
    fn test_json_database_operations() {
        let mut db = Database::open_in_memory().unwrap();

        // 创建资产
        let asset = Asset::new("测试股票", AssetType::Stock, 10000.0);
        db.create_asset(&asset).unwrap();

        // 获取资产
        let loaded = db.get_asset(asset.id).unwrap().unwrap();
        assert_eq!(loaded.name, "测试股票");
        assert_eq!(loaded.value, 10000.0);

        // 列出资产
        let assets = db.list_assets().unwrap();
        assert_eq!(assets.len(), 1);

        // 获取摘要
        let summary = db.get_summary().unwrap();
        assert_eq!(summary.total_value, 10000.0);
        assert_eq!(summary.asset_count, 1);

        // 搜索
        let results = db.search_assets("股票").unwrap();
        assert_eq!(results.len(), 1);

        let results = db.search_assets("不存在").unwrap();
        assert_eq!(results.len(), 0);

        // 更新
        let mut updated = loaded.clone();
        updated.update_value(20000.0);
        db.update_asset(&updated).unwrap();
        let reloaded = db.get_asset(asset.id).unwrap().unwrap();
        assert_eq!(reloaded.value, 20000.0);

        // 删除
        db.delete_asset(asset.id).unwrap();
        let deleted = db.get_asset(asset.id).unwrap();
        assert!(deleted.is_none());
    }

    #[test]
    fn test_settings() {
        let mut db = Database::open_in_memory().unwrap();

        db.set_setting("theme", "dark").unwrap();
        let val = db.get_setting("theme").unwrap();
        assert_eq!(val, Some("dark".to_string()));

        let missing = db.get_setting("nonexistent").unwrap();
        assert!(missing.is_none());
    }

    #[test]
    fn test_list_by_type() {
        let mut db = Database::open_in_memory().unwrap();

        db.create_asset(&Asset::new("股票A", AssetType::Stock, 5000.0)).unwrap();
        db.create_asset(&Asset::new("现金", AssetType::Cash, 3000.0)).unwrap();
        db.create_asset(&Asset::new("股票B", AssetType::Stock, 8000.0)).unwrap();

        let stocks = db.list_assets_by_type(&AssetType::Stock).unwrap();
        assert_eq!(stocks.len(), 2);

        let cash = db.list_assets_by_type(&AssetType::Cash).unwrap();
        assert_eq!(cash.len(), 1);
    }
}
