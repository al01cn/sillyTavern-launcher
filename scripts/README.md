# 构建脚本说明

## sync-version.js - 版本信息同步脚本

### 功能

在构建前自动同步 `package.json` 中的版本信息到所有相关配置文件，确保版本号一致性。

### 同步的文件

1. **src-tauri/tauri.conf.json**
   - `version`: 应用版本号
   - `productName`: 产品名称

2. **src-tauri/Cargo.toml**
   - `version`: Rust 包版本号
   - `description`: 项目描述

> 注意：
> - `src/lib/config.ts` 直接从 `package.json` 导入版本信息，无需同步
> - NSIS 安装程序使用 hooks 机制，不需要自定义模板，无需同步版本信息

### 使用方法

#### 自动执行（推荐）

构建时会自动执行同步：

```bash
# 构建 Tauri 应用（会自动同步版本）
bun run tauri:build

# 或者构建前端（会自动同步版本）
bun run build
```

#### 手动执行

如果需要手动同步版本信息：

```bash
bun run sync-version
```

### 版本更新流程

1. **修改版本号**
   
   只需要修改 `package.json` 中的 `version` 字段：
   
   ```json
   {
     "version": "0.2.0"
   }
   ```

2. **运行同步脚本**
   
   ```bash
   bun run sync-version
   ```
   
   或者直接构建（会自动同步）：
   
   ```bash
   bun run tauri:build
   ```

3. **验证同步结果**
   
   脚本会输出同步状态：
   
   ```
   🔄 开始同步版本信息...
   📦 项目名称: sillyTavern-launcher-gui
   🏷️  版本号: 0.2.0
   📝 描述: SillyTavern Launcher GUI...
   ✅ 已同步到 tauri.conf.json
   ✅ 已同步到 Cargo.toml
   ✅ 已同步到 installer.nsi
   ✅ 已同步到 config.ts
   🎉 版本信息同步完成！
   ```

### 注意事项

1. **单一数据源**
   - `package.json` 是版本信息的唯一数据源
   - 不要手动修改其他文件中的版本号

2. **构建前同步**
   - 脚本已配置为构建前自动执行
   - 确保每次构建都使用最新的版本信息

3. **错误处理**
   - 如果同步失败，脚本会退出并显示错误信息
   - 检查文件路径和格式是否正确

### 自定义配置

如需修改产品名称、发布者等信息：

1. **网站地址**：修改 `package.json` 中的 `homepage` 字段
2. **产品名称和发布者**：编辑 `scripts/sync-version.js` 中的常量：

```javascript
const productName = 'SillyTavern Launcher GUI';
const publisher = 'SillyTavern Launcher';
```

### 故障排除

**问题：同步失败**

- 检查文件是否存在
- 检查文件格式是否正确
- 查看错误信息中的具体原因

**问题：版本号不一致**

- 运行 `bun run sync-version` 手动同步
- 检查 `package.json` 中的版本号是否正确

**问题：构建时没有自动同步**

- 检查 `package.json` 中的 `prebuild` 和 `tauri:build` 脚本
- 确保使用正确的构建命令
