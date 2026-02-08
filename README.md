# Asset Manager

一个轻量级、插件化的资产管理应用程序，使用 Rust 和 Lua 构建。

## 特性

- 🦀 **Rust 核心** - 高性能、内存安全
- 🌙 **Lua 插件系统** - 灵活扩展功能
- 💾 **本地存储** - SQLite 数据库，数据完全本地化
- 🖥️ **跨平台** - PC端使用 Tauri，未来支持移动端
- 🪶 **轻量级** - 最小化依赖，快速启动

## 架构

```
asset-manager/
├── core/                 # 核心业务逻辑库
│   ├── src/
│   │   ├── asset/       # 资产管理模型
│   │   ├── plugin/      # Lua 插件系统
│   │   └── storage/     # SQLite 存储层
├── src-tauri/           # Tauri PC 应用
├── plugins/             # Lua 插件目录
└── ui/                  # 前端界面
```

## 开发环境要求

- Rust 1.75+
- Node.js 18+ (用于前端开发)
- Tauri CLI

## 快速开始

```bash
# 安装 Tauri CLI
cargo install tauri-cli

# 开发模式运行
cargo tauri dev

# 构建发布版本
cargo tauri build
```

## 插件开发

在 `plugins/` 目录下创建 Lua 插件：

```lua
-- plugins/my_plugin/init.lua
local plugin = {}

plugin.name = "My Plugin"
plugin.version = "1.0.0"

function plugin.on_load()
    print("Plugin loaded!")
end

function plugin.on_asset_create(asset)
    -- 自定义资产创建逻辑
end

return plugin
```

## License

MIT License
