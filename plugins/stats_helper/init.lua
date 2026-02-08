-- 示例插件：资产统计
-- 这是一个示例插件，展示如何编写 Asset Manager 的 Lua 插件

local plugin = {}

-- 插件元信息
plugin.name = "资产统计助手"
plugin.version = "1.0.0"
plugin.author = "Asset Manager Team"
plugin.description = "提供额外的资产统计和分析功能"

-- 内部状态
local total_created = 0
local total_deleted = 0

-- 插件加载时调用
function plugin.on_load()
    log("[资产统计] 插件已加载")
end

-- 插件卸载时调用
function plugin.on_unload()
    log("[资产统计] 插件已卸载")
    log(string.format("[资产统计] 本次会话：创建 %d 个资产，删除 %d 个资产", 
        total_created, total_deleted))
end

-- 应用启动时调用
function plugin.on_app_started()
    log("[资产统计] 应用已启动")
end

-- 应用关闭时调用
function plugin.on_app_closing()
    log("[资产统计] 应用即将关闭")
end

-- 资产创建时调用
-- @param asset_json 资产数据的 JSON 字符串
function plugin.on_asset_created(asset_json)
    total_created = total_created + 1
    log(string.format("[资产统计] 新资产创建，总计创建: %d", total_created))
    
    -- 解析 JSON（如果需要处理具体数据）
    -- local ok, asset = pcall(json.decode, asset_json)
    -- if ok then
    --     log("资产名称: " .. (asset.name or "未知"))
    -- end
end

-- 资产更新时调用
function plugin.on_asset_updated(asset_json)
    log("[资产统计] 资产已更新")
end

-- 资产删除时调用
function plugin.on_asset_deleted(asset_id)
    total_deleted = total_deleted + 1
    log(string.format("[资产统计] 资产已删除 (ID: %s)，总计删除: %d", 
        tostring(asset_id), total_deleted))
end

-- 自定义函数：获取统计摘要
function plugin.get_stats()
    return {
        created = total_created,
        deleted = total_deleted
    }
end

return plugin
