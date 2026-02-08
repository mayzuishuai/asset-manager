//! 插件加载器

use super::{PluginError, PluginInfo};
use mlua::{Lua, Result as LuaResult, Table, Value};
use std::fs;
use std::path::Path;
use tracing::{debug, info, warn};

/// 插件加载器
pub struct PluginLoader {
    lua: Lua,
}

impl PluginLoader {
    /// 创建新的插件加载器
    pub fn new() -> LuaResult<Self> {
        let lua = Lua::new();
        
        // 设置安全的 Lua 环境
        Self::setup_sandbox(&lua)?;
        
        Ok(Self { lua })
    }

    /// 设置沙箱环境，限制危险操作
    fn setup_sandbox(lua: &Lua) -> LuaResult<()> {
        // 移除危险函数
        let globals = lua.globals();
        globals.set("os", Value::Nil)?;  // 移除 os 库（安全考虑）
        globals.set("io", Value::Nil)?;  // 移除 io 库（安全考虑）
        globals.set("loadfile", Value::Nil)?;
        globals.set("dofile", Value::Nil)?;
        
        // 添加安全的日志函数
        let log_fn = lua.create_function(|_, msg: String| {
            info!(target: "plugin", "{}", msg);
            Ok(())
        })?;
        globals.set("log", log_fn)?;
        
        // 添加安全的打印函数
        let print_fn = lua.create_function(|_, args: mlua::Variadic<Value>| {
            let msg: Vec<String> = args
                .iter()
                .map(|v| format!("{:?}", v))
                .collect();
            debug!(target: "plugin", "{}", msg.join("\t"));
            Ok(())
        })?;
        globals.set("print", print_fn)?;
        
        Ok(())
    }

    /// 从目录加载插件
    pub fn load_from_dir(&self, plugin_dir: &Path) -> Result<PluginInfo, PluginError> {
        let init_file = plugin_dir.join("init.lua");
        
        if !init_file.exists() {
            return Err(PluginError::NotFound(format!(
                "Plugin init.lua not found in {:?}",
                plugin_dir
            )));
        }

        let code = fs::read_to_string(&init_file)?;
        
        self.load_plugin_code(&code, plugin_dir)
    }

    /// 加载插件代码
    fn load_plugin_code(&self, code: &str, plugin_dir: &Path) -> Result<PluginInfo, PluginError> {
        // 执行插件代码
        let plugin_table: Table = self.lua.load(code).eval().map_err(|e| {
            PluginError::LoadError(format!("Failed to load plugin: {}", e))
        })?;

        // 读取插件元信息
        let name: String = plugin_table
            .get("name")
            .unwrap_or_else(|_| "Unknown".to_string());
        
        let version: String = plugin_table
            .get("version")
            .unwrap_or_else(|_| "0.0.0".to_string());
        
        let author: Option<String> = plugin_table.get("author").ok();
        let description: Option<String> = plugin_table.get("description").ok();

        info!("Loaded plugin: {} v{}", name, version);

        Ok(PluginInfo {
            name,
            version,
            author,
            description,
            path: plugin_dir.to_path_buf(),
            enabled: true,
        })
    }

    /// 调用插件函数
    pub fn call_function<'a, A, R>(&'a self, func_name: &str, args: A) -> Result<R, PluginError>
    where
        A: mlua::IntoLuaMulti,
        R: mlua::FromLuaMulti + 'a,
    {
        let globals = self.lua.globals();
        
        if let Ok(func) = globals.get::<mlua::Function>(func_name) {
            Ok(func.call(args)?)
        } else {
            warn!("Function '{}' not found", func_name);
            Err(PluginError::NotFound(func_name.to_string()))
        }
    }

    /// 获取 Lua 实例引用
    pub fn lua(&self) -> &Lua {
        &self.lua
    }

    /// 注册 Rust 函数到 Lua
    pub fn register_function<F, A, R>(&self, name: &str, func: F) -> LuaResult<()>
    where
        F: Fn(&Lua, A) -> LuaResult<R> + Send + 'static,
        A: mlua::FromLuaMulti,
        R: mlua::IntoLuaMulti,
    {
        let lua_func = self.lua.create_function(func)?;
        self.lua.globals().set(name, lua_func)?;
        Ok(())
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new().expect("Failed to create Lua VM")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_loader_creation() {
        let loader = PluginLoader::new();
        assert!(loader.is_ok());
    }

    #[test]
    fn test_simple_lua_code() {
        let loader = PluginLoader::new().unwrap();
        let result: i32 = loader.lua().load("return 1 + 1").eval().unwrap();
        assert_eq!(result, 2);
    }
}
