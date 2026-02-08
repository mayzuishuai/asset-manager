//! SQLite 数据库实现

use super::StorageError;
use crate::asset::{Asset, AssetSummary, AssetTransaction, AssetType, Currency, TransactionType};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::fs;
use std::path::Path;
use tracing::info;
use uuid::Uuid;

/// SQLite 数据库
pub struct Database {
    conn: Connection,
}

impl Database {
    /// 打开或创建数据库
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let path = path.as_ref();
        
        // 确保父目录存在
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;
        let db = Self { conn };
        
        db.init_schema()?;
        info!("Database opened: {:?}", path);
        
        Ok(db)
    }

    /// 创建内存数据库（用于测试）
    pub fn open_in_memory() -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// 初始化数据库表结构
    fn init_schema(&self) -> Result<(), StorageError> {
        self.conn.execute_batch(
            r#"
            -- 资产表
            CREATE TABLE IF NOT EXISTS assets (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                asset_type TEXT NOT NULL,
                value REAL NOT NULL DEFAULT 0,
                currency TEXT NOT NULL DEFAULT 'CNY',
                description TEXT,
                tags TEXT,
                metadata TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            -- 交易记录表
            CREATE TABLE IF NOT EXISTS transactions (
                id TEXT PRIMARY KEY,
                asset_id TEXT NOT NULL,
                transaction_type TEXT NOT NULL,
                amount_before REAL NOT NULL,
                amount_after REAL NOT NULL,
                note TEXT,
                timestamp TEXT NOT NULL,
                FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
            );

            -- 应用设置表
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            -- 创建索引
            CREATE INDEX IF NOT EXISTS idx_assets_type ON assets(asset_type);
            CREATE INDEX IF NOT EXISTS idx_assets_created ON assets(created_at);
            CREATE INDEX IF NOT EXISTS idx_transactions_asset ON transactions(asset_id);
            CREATE INDEX IF NOT EXISTS idx_transactions_time ON transactions(timestamp);
            "#,
        )?;

        Ok(())
    }

    // ============ 资产操作 ============

    /// 创建资产
    pub fn create_asset(&self, asset: &Asset) -> Result<(), StorageError> {
        self.conn.execute(
            r#"
            INSERT INTO assets (id, name, asset_type, value, currency, description, tags, metadata, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            "#,
            params![
                asset.id.to_string(),
                asset.name,
                asset.asset_type.as_str(),
                asset.value,
                serde_json::to_string(&asset.currency)?,
                asset.description,
                serde_json::to_string(&asset.tags)?,
                asset.metadata.to_string(),
                asset.created_at.to_rfc3339(),
                asset.updated_at.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    /// 获取资产
    pub fn get_asset(&self, id: Uuid) -> Result<Option<Asset>, StorageError> {
        let result = self.conn.query_row(
            "SELECT * FROM assets WHERE id = ?1",
            params![id.to_string()],
            |row| self.row_to_asset(row),
        ).optional()?;

        Ok(result)
    }

    /// 获取所有资产
    pub fn list_assets(&self) -> Result<Vec<Asset>, StorageError> {
        let mut stmt = self.conn.prepare("SELECT * FROM assets ORDER BY created_at DESC")?;
        
        let assets = stmt
            .query_map([], |row| self.row_to_asset(row))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(assets)
    }

    /// 按类型获取资产
    pub fn list_assets_by_type(&self, asset_type: &AssetType) -> Result<Vec<Asset>, StorageError> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM assets WHERE asset_type = ?1 ORDER BY created_at DESC"
        )?;
        
        let assets = stmt
            .query_map(params![asset_type.as_str()], |row| self.row_to_asset(row))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(assets)
    }

    /// 更新资产
    pub fn update_asset(&self, asset: &Asset) -> Result<(), StorageError> {
        let rows = self.conn.execute(
            r#"
            UPDATE assets SET
                name = ?2,
                asset_type = ?3,
                value = ?4,
                currency = ?5,
                description = ?6,
                tags = ?7,
                metadata = ?8,
                updated_at = ?9
            WHERE id = ?1
            "#,
            params![
                asset.id.to_string(),
                asset.name,
                asset.asset_type.as_str(),
                asset.value,
                serde_json::to_string(&asset.currency)?,
                asset.description,
                serde_json::to_string(&asset.tags)?,
                asset.metadata.to_string(),
                asset.updated_at.to_rfc3339(),
            ],
        )?;

        if rows == 0 {
            return Err(StorageError::NotFound(asset.id.to_string()));
        }

        Ok(())
    }

    /// 删除资产
    pub fn delete_asset(&self, id: Uuid) -> Result<(), StorageError> {
        let rows = self.conn.execute(
            "DELETE FROM assets WHERE id = ?1",
            params![id.to_string()],
        )?;

        if rows == 0 {
            return Err(StorageError::NotFound(id.to_string()));
        }

        Ok(())
    }

    /// 搜索资产
    pub fn search_assets(&self, query: &str) -> Result<Vec<Asset>, StorageError> {
        let pattern = format!("%{}%", query);
        let mut stmt = self.conn.prepare(
            r#"
            SELECT * FROM assets 
            WHERE name LIKE ?1 OR description LIKE ?1 OR tags LIKE ?1
            ORDER BY created_at DESC
            "#
        )?;
        
        let assets = stmt
            .query_map(params![pattern], |row| self.row_to_asset(row))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(assets)
    }

    /// 从数据库行解析资产
    fn row_to_asset(&self, row: &rusqlite::Row) -> rusqlite::Result<Asset> {
        let id_str: String = row.get("id")?;
        let asset_type_str: String = row.get("asset_type")?;
        let currency_str: String = row.get("currency")?;
        let tags_str: String = row.get("tags")?;
        let metadata_str: String = row.get("metadata")?;
        let created_str: String = row.get("created_at")?;
        let updated_str: String = row.get("updated_at")?;

        Ok(Asset {
            id: Uuid::parse_str(&id_str).unwrap_or_default(),
            name: row.get("name")?,
            asset_type: self.parse_asset_type(&asset_type_str),
            value: row.get("value")?,
            currency: serde_json::from_str(&currency_str).unwrap_or_default(),
            description: row.get("description")?,
            tags: serde_json::from_str(&tags_str).unwrap_or_default(),
            metadata: serde_json::from_str(&metadata_str).unwrap_or_default(),
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }

    fn parse_asset_type(&self, s: &str) -> AssetType {
        match s {
            "cash" => AssetType::Cash,
            "bank_deposit" => AssetType::BankDeposit,
            "stock" => AssetType::Stock,
            "fund" => AssetType::Fund,
            "bond" => AssetType::Bond,
            "real_estate" => AssetType::RealEstate,
            "vehicle" => AssetType::Vehicle,
            "crypto" => AssetType::Crypto,
            "precious_metal" => AssetType::PreciousMetal,
            other => AssetType::Other(other.to_string()),
        }
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
    pub fn add_transaction(&self, transaction: &AssetTransaction) -> Result<(), StorageError> {
        self.conn.execute(
            r#"
            INSERT INTO transactions (id, asset_id, transaction_type, amount_before, amount_after, note, timestamp)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                transaction.id.to_string(),
                transaction.asset_id.to_string(),
                format!("{:?}", transaction.transaction_type),
                transaction.amount_before,
                transaction.amount_after,
                transaction.note,
                transaction.timestamp.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    /// 获取资产的交易历史
    pub fn get_transactions(&self, asset_id: Uuid) -> Result<Vec<AssetTransaction>, StorageError> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM transactions WHERE asset_id = ?1 ORDER BY timestamp DESC"
        )?;
        
        let transactions = stmt
            .query_map(params![asset_id.to_string()], |row| {
                let id_str: String = row.get("id")?;
                let asset_id_str: String = row.get("asset_id")?;
                let type_str: String = row.get("transaction_type")?;
                let timestamp_str: String = row.get("timestamp")?;

                Ok(AssetTransaction {
                    id: Uuid::parse_str(&id_str).unwrap_or_default(),
                    asset_id: Uuid::parse_str(&asset_id_str).unwrap_or_default(),
                    transaction_type: Self::parse_transaction_type(&type_str),
                    amount_before: row.get("amount_before")?,
                    amount_after: row.get("amount_after")?,
                    note: row.get("note")?,
                    timestamp: DateTime::parse_from_rfc3339(&timestamp_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(transactions)
    }

    fn parse_transaction_type(s: &str) -> TransactionType {
        match s {
            "Buy" => TransactionType::Buy,
            "Sell" => TransactionType::Sell,
            "ValueChange" => TransactionType::ValueChange,
            "Income" => TransactionType::Income,
            "Expense" => TransactionType::Expense,
            "Transfer" => TransactionType::Transfer,
            _ => TransactionType::ValueChange,
        }
    }

    // ============ 设置 ============

    /// 保存设置
    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), StorageError> {
        self.conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    /// 获取设置
    pub fn get_setting(&self, key: &str) -> Result<Option<String>, StorageError> {
        let result = self.conn.query_row(
            "SELECT value FROM settings WHERE key = ?1",
            params![key],
            |row| row.get(0),
        ).optional()?;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_operations() {
        let db = Database::open_in_memory().unwrap();

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
    }
}
