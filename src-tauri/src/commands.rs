//! Tauri 命令处理

use crate::AppState;
use asset_manager_core::{
    asset::{Asset, AssetSummary, AssetType, Currency},
    plugin::PluginEvent,
};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

/// 创建资产的请求参数
#[derive(Debug, Deserialize)]
pub struct CreateAssetRequest {
    pub name: String,
    pub asset_type: String,
    pub value: f64,
    pub currency: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// 更新资产的请求参数
#[derive(Debug, Deserialize)]
pub struct UpdateAssetRequest {
    pub id: String,
    pub name: Option<String>,
    pub value: Option<f64>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

/// 插件信息响应
#[derive(Debug, Serialize)]
pub struct PluginInfoResponse {
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub enabled: bool,
}

// ============ 资产命令 ============

/// 获取所有资产
#[tauri::command]
pub fn get_assets(state: State<'_, AppState>) -> Result<Vec<Asset>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.list_assets().map_err(|e| e.to_string())
}

/// 获取单个资产
#[tauri::command]
pub fn get_asset(state: State<'_, AppState>, id: String) -> Result<Option<Asset>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    db.get_asset(uuid).map_err(|e| e.to_string())
}

/// 创建资产
#[tauri::command]
pub fn create_asset(
    state: State<'_, AppState>,
    request: CreateAssetRequest,
) -> Result<Asset, String> {
    let asset_type = parse_asset_type(&request.asset_type);
    let currency = request
        .currency
        .as_ref()
        .map(|c| parse_currency(c))
        .unwrap_or_default();

    let mut asset = Asset::new(request.name, asset_type, request.value)
        .with_currency(currency);

    if let Some(desc) = request.description {
        asset = asset.with_description(desc);
    }

    if let Some(tags) = request.tags {
        asset = asset.with_tags(tags);
    }

    // 保存到数据库
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.create_asset(&asset).map_err(|e| e.to_string())?;
    }

    // 触发插件事件
    {
        let pm = state.plugin_manager.lock().map_err(|e| e.to_string())?;
        pm.broadcast_event(&PluginEvent::AssetCreated(asset.clone()));
    }

    Ok(asset)
}

/// 更新资产
#[tauri::command]
pub fn update_asset(
    state: State<'_, AppState>,
    request: UpdateAssetRequest,
) -> Result<Asset, String> {
    let uuid = Uuid::parse_str(&request.id).map_err(|e| e.to_string())?;

    let db = state.db.lock().map_err(|e| e.to_string())?;
    
    let mut asset = db
        .get_asset(uuid)
        .map_err(|e| e.to_string())?
        .ok_or("Asset not found")?;

    if let Some(name) = request.name {
        asset.name = name;
    }
    if let Some(value) = request.value {
        asset.update_value(value);
    }
    if let Some(desc) = request.description {
        asset.description = Some(desc);
    }
    if let Some(tags) = request.tags {
        asset.tags = tags;
    }

    db.update_asset(&asset).map_err(|e| e.to_string())?;

    // 触发插件事件
    drop(db);
    {
        let pm = state.plugin_manager.lock().map_err(|e| e.to_string())?;
        pm.broadcast_event(&PluginEvent::AssetUpdated(asset.clone()));
    }

    Ok(asset)
}

/// 删除资产
#[tauri::command]
pub fn delete_asset(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;

    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        db.delete_asset(uuid).map_err(|e| e.to_string())?;
    }

    // 触发插件事件
    {
        let pm = state.plugin_manager.lock().map_err(|e| e.to_string())?;
        pm.broadcast_event(&PluginEvent::AssetDeleted(uuid));
    }

    Ok(())
}

/// 搜索资产
#[tauri::command]
pub fn search_assets(state: State<'_, AppState>, query: String) -> Result<Vec<Asset>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.search_assets(&query).map_err(|e| e.to_string())
}

/// 获取资产摘要
#[tauri::command]
pub fn get_summary(state: State<'_, AppState>) -> Result<AssetSummary, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_summary().map_err(|e| e.to_string())
}

// ============ 插件命令 ============

/// 获取插件列表
#[tauri::command]
pub fn get_plugins(state: State<'_, AppState>) -> Result<Vec<PluginInfoResponse>, String> {
    let pm = state.plugin_manager.lock().map_err(|e| e.to_string())?;
    
    let plugins = pm
        .list_plugins()
        .iter()
        .map(|p| PluginInfoResponse {
            name: p.name.clone(),
            version: p.version.clone(),
            author: p.author.clone(),
            description: p.description.clone(),
            enabled: p.enabled,
        })
        .collect();

    Ok(plugins)
}

/// 重新加载插件
#[tauri::command]
pub fn reload_plugins(state: State<'_, AppState>) -> Result<Vec<PluginInfoResponse>, String> {
    let mut pm = state.plugin_manager.lock().map_err(|e| e.to_string())?;
    
    let loaded = pm.load_all().map_err(|e| e.to_string())?;
    
    let plugins = loaded
        .iter()
        .map(|p| PluginInfoResponse {
            name: p.name.clone(),
            version: p.version.clone(),
            author: p.author.clone(),
            description: p.description.clone(),
            enabled: p.enabled,
        })
        .collect();

    Ok(plugins)
}

/// 设置插件启用状态
#[tauri::command]
pub fn set_plugin_enabled(
    state: State<'_, AppState>,
    name: String,
    enabled: bool,
) -> Result<(), String> {
    let mut pm = state.plugin_manager.lock().map_err(|e| e.to_string())?;
    pm.set_plugin_enabled(&name, enabled).map_err(|e| e.to_string())
}

// ============ 辅助函数 ============

fn parse_asset_type(s: &str) -> AssetType {
    match s.to_lowercase().as_str() {
        "cash" => AssetType::Cash,
        "bank_deposit" | "bank" => AssetType::BankDeposit,
        "stock" => AssetType::Stock,
        "fund" => AssetType::Fund,
        "bond" => AssetType::Bond,
        "real_estate" | "property" => AssetType::RealEstate,
        "vehicle" | "car" => AssetType::Vehicle,
        "crypto" | "cryptocurrency" => AssetType::Crypto,
        "precious_metal" | "gold" | "silver" => AssetType::PreciousMetal,
        other => AssetType::Other(other.to_string()),
    }
}

fn parse_currency(s: &str) -> Currency {
    match s.to_uppercase().as_str() {
        "CNY" | "RMB" => Currency::CNY,
        "USD" => Currency::USD,
        "EUR" => Currency::EUR,
        "GBP" => Currency::GBP,
        "JPY" => Currency::JPY,
        "HKD" => Currency::HKD,
        other => Currency::Other(other.to_string()),
    }
}
