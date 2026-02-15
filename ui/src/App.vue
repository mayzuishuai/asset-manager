<script setup>
import { ref, onMounted, computed } from 'vue'

// Tauri API
const invoke = window.__TAURI__?.core?.invoke

// 状态
const assets = ref([])
const summary = ref({ total_value: 0, asset_count: 0, by_type: {}, by_currency: {} })
const plugins = ref([])
const loading = ref(false)
const currentView = ref('dashboard')  // dashboard, assets, plugins, add
const searchQuery = ref('')
const showAddModal = ref(false)
const showDeleteModal = ref(false)
const pendingDeleteAsset = ref(null)
const addMode = ref('asset')
const selectedTags = ref([])
const tagInput = ref('')
const tagInputFocused = ref(false)
const tagInputRef = ref(null)
const tagLibraries = ref({})
let tagBlurTimer = null

// 新资产表单
const newAsset = ref({
  name: '',
  asset_type: '',
  value: null,
  currency: '',
  description: '',
  tags: []
})

// 资产类型标签
const ASSET_TYPE_LABELS = {
  cash: '现金',
  bank_deposit: '银行存款',
  stock: '股票',
  fund: '基金',
  bond: '债券',
  real_estate: '房产',
  vehicle: '车辆',
  crypto: '加密货币',
  precious_metal: '贵金属',
  other: '其他'
}

// 负债类型标签
const LIABILITY_TYPE_LABELS = {
  huabei: '花呗',
  credit_card: '信用卡',
  car_loan: '车贷',
  mortgage: '房贷',
  bank_loan: '银行贷款',
  other: '其他'
}

// 币种定义
const CURRENCY_DEFINITIONS = {
  CNY: { code: 'CNY', label: '人民币 (CNY)', symbol: '¥' },
  USD: { code: 'USD', label: '美元 (USD)', symbol: '$' },
  EUR: { code: 'EUR', label: '欧元 (EUR)', symbol: '€' },
  HKD: { code: 'HKD', label: '港币 (HKD)', symbol: 'HK$' }
}

const CURRENCY_OPTIONS = Object.values(CURRENCY_DEFINITIONS).map(item => ({
  code: item.code,
  label: item.label
}))

const CURRENCY_SYMBOL_BY_CODE = Object.values(CURRENCY_DEFINITIONS).reduce((result, item) => {
  result[item.code] = item.symbol
  return result
}, {})

// 计算属性
const filteredAssets = computed(() => {
  if (!searchQuery.value) return assets.value
  const query = searchQuery.value.toLowerCase()
  return assets.value.filter(a => 
    a.name.toLowerCase().includes(query) ||
    (a.description && a.description.toLowerCase().includes(query))
  )
})

const currentTagLibrary = computed(() => {
  const typeKey = newAsset.value.asset_type
  return tagLibraries.value[typeKey] || { tags: [], recents: [] }
})

const filteredTagSuggestions = computed(() => {
  const query = tagInput.value.trim().toLowerCase()
  const library = currentTagLibrary.value
  const baseList = query
    ? library.tags.filter(tag => tag.toLowerCase().includes(query))
    : library.recents
  return baseList.filter(tag => !selectedTags.value.includes(tag)).slice(0, 8)
})

const showTagDropdown = computed(() => {
  return tagInputFocused.value && filteredTagSuggestions.value.length > 0
})

// API 调用
async function loadAssets() {
  if (!invoke) return
  loading.value = true
  try {
    assets.value = await invoke('get_assets')
    summary.value = await invoke('get_summary')
    mergeTagLibrariesFromAssets()
  } catch (e) {
    console.error('Failed to load assets:', e)
  } finally {
    loading.value = false
  }
}

async function loadPlugins() {
  if (!invoke) return
  try {
    plugins.value = await invoke('get_plugins')
  } catch (e) {
    console.error('Failed to load plugins:', e)
  }
}

async function createAsset() {
  if (!invoke || !newAsset.value.name || !newAsset.value.asset_type || !newAsset.value.currency) return
  try {
    await invoke('create_asset', {
      request: {
        name: newAsset.value.name,
        asset_type: newAsset.value.asset_type,
        value: newAsset.value.value,
        currency: newAsset.value.currency,
        description: newAsset.value.description || null,
        tags: selectedTags.value.length > 0 ? selectedTags.value : null
      }
    })
    
    // 重置表单
    newAsset.value = { name: '', asset_type: '', value: null, currency: '', description: '', tags: [] }
    selectedTags.value = []
    tagInput.value = ''
    showAddModal.value = false
    await loadAssets()
  } catch (e) {
    console.error('Failed to create asset:', e)
    alert('创建失败: ' + e)
  }
}

function openAddModal(mode) {
  addMode.value = mode
  showAddModal.value = true
  newAsset.value.value = null
  newAsset.value.asset_type = ''
  newAsset.value.currency = ''
}

function requestDeleteAsset(asset) {
  pendingDeleteAsset.value = asset
  showDeleteModal.value = true
}

async function confirmDeleteAsset() {
  if (!invoke || !pendingDeleteAsset.value) return
  try {
    await invoke('delete_asset', { id: pendingDeleteAsset.value.id })
    await loadAssets()
  } catch (e) {
    console.error('Failed to delete asset:', e)
  } finally {
    showDeleteModal.value = false
    pendingDeleteAsset.value = null
  }
}

function cancelDeleteAsset() {
  showDeleteModal.value = false
  pendingDeleteAsset.value = null
}

async function togglePlugin(name, enabled) {
  if (!invoke) return
  try {
    await invoke('set_plugin_enabled', { name, enabled: !enabled })
    await loadPlugins()
  } catch (e) {
    console.error('Failed to toggle plugin:', e)
  }
}

function formatValue(value, currency) {
  const symbol = CURRENCY_SYMBOL_BY_CODE[currency] || CURRENCY_SYMBOL_BY_CODE['CNY']
  return `${symbol}${value.toLocaleString('zh-CN', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`
}

function getAssetTypeName(type) {
  if (typeof type === 'object' && type.Other) {
    return type.Other
  }
  const key = typeof type === 'string' ? type.toLowerCase() : 'other'
  return ASSET_TYPE_LABELS[key] || type
}

function getAssetTypeKey(type) {
  if (typeof type === 'string') return type
  if (type && type.Other) return 'other'
  return 'other'
}

function ensureTagLibrary(typeKey) {
  if (!tagLibraries.value[typeKey]) {
    tagLibraries.value[typeKey] = { tags: [], recents: [] }
  }
}

function normalizeTag(tag) {
  if (!tag) return ''
  return tag.toString().trim()
}

function addTagToLibrary(typeKey, tag, makeRecent = false) {
  const cleanTag = normalizeTag(tag)
  if (!cleanTag) return
  ensureTagLibrary(typeKey)
  const library = tagLibraries.value[typeKey]
  if (!library.tags.includes(cleanTag)) {
    library.tags.push(cleanTag)
  }
  if (makeRecent) {
    library.recents = library.recents.filter(item => item !== cleanTag)
    library.recents.unshift(cleanTag)
    library.recents = library.recents.slice(0, 8)
  }
}

function mergeTagLibrariesFromAssets() {
  assets.value.forEach(asset => {
    const typeKey = getAssetTypeKey(asset.asset_type)
    const tags = Array.isArray(asset.tags) ? asset.tags : []
    ensureTagLibrary(typeKey)
    const allowRecent = tagLibraries.value[typeKey].recents.length === 0
    tags.forEach(tag => addTagToLibrary(typeKey, tag, allowRecent))
  })
}

function focusTagInput() {
  tagInputRef.value?.focus()
}

function handleTagInputFocus() {
  if (tagBlurTimer) {
    clearTimeout(tagBlurTimer)
    tagBlurTimer = null
  }
  tagInputFocused.value = true
}

function handleTagInputBlur() {
  if (tagBlurTimer) {
    clearTimeout(tagBlurTimer)
  }
  tagBlurTimer = setTimeout(() => {
    tagInputFocused.value = false
    commitTagInput()
  }, 120)
}

function addTag(tag) {
  const cleanTag = normalizeTag(tag)
  if (!cleanTag || selectedTags.value.includes(cleanTag)) return
  selectedTags.value.push(cleanTag)
  addTagToLibrary(newAsset.value.asset_type, cleanTag, true)
}

function removeTag(tag) {
  selectedTags.value = selectedTags.value.filter(item => item !== tag)
}

function commitTagInput() {
  if (!tagInput.value) return
  addTag(tagInput.value)
  tagInput.value = ''
}

function selectTag(tag) {
  addTag(tag)
  tagInput.value = ''
  focusTagInput()
}

function handleTagInputKeydown(event) {
  if (event.key === 'Enter') {
    event.preventDefault()
    commitTagInput()
    return
  }
  if (event.key === 'Backspace' && !tagInput.value && selectedTags.value.length > 0) {
    removeTag(selectedTags.value[selectedTags.value.length - 1])
  }
}

onMounted(() => {
  loadAssets()
  loadPlugins()
})
</script>

<template>
  <div class="app">
    <!-- 侧边栏 -->
    <aside class="sidebar">
      <div class="logo">
        <h1>资产管理器</h1>
      </div>
      <nav>
        <a href="#" :class="{ active: currentView === 'dashboard' }" @click.prevent="currentView = 'dashboard'">
          仪表盘
        </a>
        <a href="#" :class="{ active: currentView === 'assets' }" @click.prevent="currentView = 'assets'">
          资产与负债
        </a>
        <a href="#" :class="{ active: currentView === 'plugins' }" @click.prevent="currentView = 'plugins'">
          插件管理
        </a>
      </nav>
    </aside>

    <!-- 主内容 -->
    <main class="main-content">
      <!-- 仪表盘 -->
      <div v-if="currentView === 'dashboard'" class="dashboard">
        <h2>资产概览</h2>
        
        <div class="stats-grid">
          <div class="stat-card total">
            <div class="stat-label">总资产</div>
            <div class="stat-value">{{ formatValue(summary.total_value, 'CNY') }}</div>
          </div>
          <div class="stat-card count">
            <div class="stat-label">资产数量</div>
            <div class="stat-value">{{ summary.asset_count }}</div>
          </div>
        </div>

        <div class="chart-section" v-if="Object.keys(summary.by_type).length > 0">
          <h3>按类型分布</h3>
          <div class="type-list">
            <div v-for="(value, type) in summary.by_type" :key="type" class="type-item">
              <span class="type-name">{{ ASSET_TYPE_LABELS[type] || type }}</span>
              <span class="type-value">{{ formatValue(value, 'CNY') }}</span>
              <div class="type-bar" :style="{ width: (value / summary.total_value * 100) + '%' }"></div>
            </div>
          </div>
        </div>
      </div>

      <!-- 资产与负债 -->
      <div v-if="currentView === 'assets'" class="assets-view">
        <div class="header-bar">
          <h2>资产与负债</h2>
          <div class="actions">
            <input 
              v-model="searchQuery" 
              type="text" 
              placeholder="搜索..." 
              class="search-input"
            />
            <button class="btn btn-primary" @click="openAddModal('asset')">+ 添加资产</button>
            <button class="btn btn-liability" @click="openAddModal('liability')">+ 添加负债</button>
          </div>
        </div>

        <div v-if="loading" class="loading">加载中...</div>
        
        <div v-else-if="filteredAssets.length === 0" class="empty">
          暂无资产与负债记录
        </div>
        
        <div v-else class="asset-list">
          <div
            v-for="asset in filteredAssets"
            :key="asset.id"
            class="asset-card"
            :class="{ liability: asset.value < 0 }"
          >
            <div class="asset-body">
              <div class="asset-header">
                <h3>{{ asset.name }}</h3>
                <span class="asset-type">{{ getAssetTypeName(asset.asset_type) }}</span>
              </div>
              <div class="asset-value">{{ formatValue(asset.value, 'CNY') }}</div>
              <p v-if="asset.description" class="asset-desc">{{ asset.description }}</p>
              <div v-if="asset.tags && asset.tags.length" class="asset-tags">
                <span v-for="tag in asset.tags" :key="tag" class="tag">{{ tag }}</span>
              </div>
            </div>
            <div class="asset-actions">
              <button class="delete-action" type="button" @click="requestDeleteAsset(asset)">
                <svg class="delete-icon" viewBox="0 0 22 22" aria-hidden="true">
                  <path d="M9 3h6l1 2h4v2H4V5h4l1-2zm1 6h2v9h-2V9zm4 0h2v9h-2V9zM7 9h2v9H7V9z" />
                </svg>
                删除
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- 插件管理 -->
      <div v-if="currentView === 'plugins'" class="plugins-view">
        <h2>插件管理</h2>
        
        <div v-if="plugins.length === 0" class="empty">
          暂无已安装的插件
        </div>
        
        <div v-else class="plugin-list">
          <div v-for="plugin in plugins" :key="plugin.name" class="plugin-card">
            <div class="plugin-info">
              <h3>{{ plugin.name }}</h3>
              <span class="plugin-version">v{{ plugin.version }}</span>
              <p v-if="plugin.description">{{ plugin.description }}</p>
              <p v-if="plugin.author" class="plugin-author">作者: {{ plugin.author }}</p>
            </div>
            <div class="plugin-toggle">
              <label class="switch">
                <input 
                  type="checkbox" 
                  :checked="plugin.enabled"
                  @change="togglePlugin(plugin.name, plugin.enabled)"
                />
                <span class="slider"></span>
              </label>
            </div>
          </div>
        </div>
      </div>
    </main>

    <!-- 添加资产/负债模态框 -->
    <div v-if="showAddModal" class="modal-overlay" @click.self="showAddModal = false">
      <div class="modal">
        <h2>{{ addMode === 'liability' ? '添加负债' : '添加资产' }}</h2>
        <form @submit.prevent="createAsset">
          <div class="form-group">
            <label>{{ addMode === 'liability' ? '负债名称 *' : '资产名称 *' }}</label>
            <input
              v-model="newAsset.name"
              type="text"
              required
              :placeholder="addMode === 'liability' ? '如：**信用卡' : '如：**银行储蓄'"
            />
          </div>
          
          <div class="form-row">
            <div class="form-group">
              <label>{{ addMode === 'liability' ? '负债类型 *' : '资产类型 *' }}</label>
              <select 
                v-model="newAsset.asset_type"
                :class="{ 'select-placeholder': !newAsset.asset_type }"
                required
              >
                <option value="" disabled hidden>请选择类型</option>
                <option
                  v-for="(name, key) in (addMode === 'liability' ? LIABILITY_TYPE_LABELS : ASSET_TYPE_LABELS)"
                  :key="key"
                  :value="key"
                >
                  {{ name }}
                </option>
              </select>
            </div>
            <div class="form-group">
              <label>币种 *</label>
              <select
                v-model="newAsset.currency"
                :class="{ 'select-placeholder': !newAsset.currency }"
                required
              >
                <option value="" disabled hidden>请选择币种</option>
                <option
                  v-for="currency in CURRENCY_OPTIONS"
                  :key="currency.code"
                  :value="currency.code"
                >
                  {{ currency.label }}
                </option>
              </select>
            </div>
          </div>
          
          <div class="form-group">
            <label>价值 *</label>
            <input
              v-model.number="newAsset.value"
              type="number"
              required
              step="0.01"
              :min="addMode === 'liability' ? -1000000000000.00 : 0.01"
              :max="addMode === 'liability' ? -0.01 : 1000000000000.00"
              :placeholder="addMode === 'liability' ? '-0.01' : '0.01'"
            />
          </div>
          
          <div class="form-group">
            <label>{{ addMode === 'liability' ? '备注' : '描述' }}</label>
            <textarea v-model="newAsset.description" placeholder="可选描述..."></textarea>
          </div>
          
          <div class="form-group">
            <label>标签</label>
            <div class="tag-input-wrapper">
              <div class="tag-input" @click="focusTagInput">
                <span v-for="tag in selectedTags" :key="tag" class="tag-chip">
                  {{ tag }}
                  <button type="button" class="tag-remove" @click.stop="removeTag(tag)">×</button>
                </span>
                <input
                  ref="tagInputRef"
                  v-model="tagInput"
                  type="text"
                  placeholder="输入标签，回车确认"
                  @focus="handleTagInputFocus"
                  @blur="handleTagInputBlur"
                  @keydown="handleTagInputKeydown"
                />
              </div>
              <ul v-if="showTagDropdown" class="tag-dropdown">
                <li
                  v-for="tag in filteredTagSuggestions"
                  :key="tag"
                  @mousedown.prevent="selectTag(tag)"
                >
                  {{ tag }}
                </li>
              </ul>
            </div>
          </div>
          
          <div class="form-actions">
            <button type="button" class="btn" @click="showAddModal = false">取消</button>
            <button type="submit" class="btn" :class="addMode === 'liability' ? 'btn-liability' : 'btn-primary'">创建</button>
          </div>
        </form>
      </div>
    </div>

    <!-- 删除确认模态框 -->
    <div v-if="showDeleteModal" class="modal-overlay modal-overlay--danger" @click.self="cancelDeleteAsset">
      <div class="modal modal-danger" role="dialog" aria-modal="true" aria-labelledby="delete-title">
        <div class="modal-danger__title-row">
          <div class="modal-danger__icon" aria-hidden="true">
            <svg viewBox="0 0 24 24" width="28" height="28" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M12 8v5" />
              <path d="M12 16h.01" />
              <circle cx="12" cy="12" r="9" />
            </svg>
          </div>
          <h2 id="delete-title">确认删除</h2>
        </div>
        <div class="modal-danger__body">
          <p>即将删除资产<span v-if="pendingDeleteAsset" class="modal-danger__asset">{{ pendingDeleteAsset.name }}</span>，此操作不可恢复。</p>
        </div>
        <div class="modal-danger__actions">
          <button type="button" class="btn btn-outline" @click="cancelDeleteAsset">取消</button>
          <button type="button" class="btn btn-danger" @click="confirmDeleteAsset">确认删除</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 此组件样式在 style.css 中定义 */
</style>
