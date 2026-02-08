//! Asset Manager - Tauri Desktop Application

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use asset_manager_core::{AppConfig, Database, PluginManager};
use std::sync::Mutex;
use tracing::info;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// 应用程序状态
pub struct AppState {
    pub db: Mutex<Database>,
    pub plugin_manager: Mutex<PluginManager>,
    pub config: AppConfig,
}

fn main() {
    // 初始化日志
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env().add_directive("asset_manager=debug".parse().unwrap()))
        .init();

    info!("Starting Asset Manager...");

    // 加载配置
    let config = AppConfig::default();

    // 初始化 JSON 存储
    let db = Database::open(&config.db_path).expect("Failed to open database");

    // 初始化插件管理器
    let mut plugin_manager = PluginManager::new(&config.plugins_dir);
    if let Err(e) = plugin_manager.load_all() {
        tracing::warn!("Failed to load plugins: {}", e);
    }

    // 构建应用状态
    let state = AppState {
        db: Mutex::new(db),
        plugin_manager: Mutex::new(plugin_manager),
        config,
    };

    // 启动 Tauri 应用
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::get_assets,
            commands::get_asset,
            commands::create_asset,
            commands::update_asset,
            commands::delete_asset,
            commands::search_assets,
            commands::get_summary,
            commands::get_plugins,
            commands::reload_plugins,
            commands::set_plugin_enabled,
        ])
        .run(tauri::generate_context!())
        .expect("Error running tauri application");
}
