//! 资产数据模型

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 资产类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    /// 现金
    Cash,
    /// 银行存款
    BankDeposit,
    /// 股票
    Stock,
    /// 基金
    Fund,
    /// 债券
    Bond,
    /// 房产
    RealEstate,
    /// 车辆
    Vehicle,
    /// 加密货币
    Crypto,
    /// 贵金属
    PreciousMetal,
    /// 其他资产
    Other(String),
}

impl AssetType {
    pub fn as_str(&self) -> &str {
        match self {
            AssetType::Cash => "cash",
            AssetType::BankDeposit => "bank_deposit",
            AssetType::Stock => "stock",
            AssetType::Fund => "fund",
            AssetType::Bond => "bond",
            AssetType::RealEstate => "real_estate",
            AssetType::Vehicle => "vehicle",
            AssetType::Crypto => "crypto",
            AssetType::PreciousMetal => "precious_metal",
            AssetType::Other(_) => "other",
        }
    }
}

/// 货币类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Currency {
    #[default]
    CNY,
    USD,
    EUR,
    GBP,
    JPY,
    HKD,
    Other(String),
}

/// 资产记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    /// 唯一标识符
    pub id: Uuid,
    /// 资产名称
    pub name: String,
    /// 资产类型
    pub asset_type: AssetType,
    /// 当前价值
    pub value: f64,
    /// 货币类型
    pub currency: Currency,
    /// 描述/备注
    pub description: Option<String>,
    /// 标签
    pub tags: Vec<String>,
    /// 自定义元数据 (JSON)
    pub metadata: serde_json::Value,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

impl Asset {
    /// 创建新资产
    pub fn new(name: impl Into<String>, asset_type: AssetType, value: f64) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            asset_type,
            value,
            currency: Currency::default(),
            description: None,
            tags: Vec::new(),
            metadata: serde_json::json!({}),
            created_at: now,
            updated_at: now,
        }
    }

    /// 设置货币类型
    pub fn with_currency(mut self, currency: Currency) -> Self {
        self.currency = currency;
        self
    }

    /// 设置描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// 添加标签
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// 设置元数据
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// 更新资产价值
    pub fn update_value(&mut self, value: f64) {
        self.value = value;
        self.updated_at = Utc::now();
    }
}

/// 资产变动记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetTransaction {
    /// 唯一标识符
    pub id: Uuid,
    /// 关联的资产ID
    pub asset_id: Uuid,
    /// 变动类型
    pub transaction_type: TransactionType,
    /// 变动前金额
    pub amount_before: f64,
    /// 变动后金额
    pub amount_after: f64,
    /// 备注
    pub note: Option<String>,
    /// 交易时间
    pub timestamp: DateTime<Utc>,
}

/// 交易类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    /// 买入/增加
    Buy,
    /// 卖出/减少
    Sell,
    /// 价值变动（如股价涨跌）
    ValueChange,
    /// 收益（如利息、分红）
    Income,
    /// 支出（如手续费）
    Expense,
    /// 转移
    Transfer,
}

/// 资产统计摘要
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AssetSummary {
    /// 总资产价值
    pub total_value: f64,
    /// 各类型资产统计
    pub by_type: std::collections::HashMap<String, f64>,
    /// 各货币资产统计
    pub by_currency: std::collections::HashMap<String, f64>,
    /// 资产数量
    pub asset_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_asset() {
        let asset = Asset::new("测试股票", AssetType::Stock, 10000.0)
            .with_currency(Currency::CNY)
            .with_description("测试资产描述")
            .with_tags(vec!["投资".to_string(), "A股".to_string()]);

        assert_eq!(asset.name, "测试股票");
        assert_eq!(asset.asset_type, AssetType::Stock);
        assert_eq!(asset.value, 10000.0);
        assert_eq!(asset.currency, Currency::CNY);
        assert!(asset.description.is_some());
        assert_eq!(asset.tags.len(), 2);
    }
}
