#!/usr/bin/env node
/**
 * 版本信息同步脚本
 * 在构建前自动同步 package.json 中的版本信息到各个配置文件
 */

import { readFileSync, writeFileSync } from 'fs';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const rootDir = join(__dirname, '..');

// 读取 package.json
const packageJson = JSON.parse(
  readFileSync(join(rootDir, 'package.json'), 'utf-8')
);

const { name, version, description } = packageJson;
const homepage = packageJson.homepage || 'https://github.com/al01cn/sillytavern-launcher';

console.log('🔄 开始同步版本信息...');
console.log(`📦 项目名称: ${name}`);
console.log(`🏷️ 版本号: ${version}`);
console.log(`📝 描述: ${description}`);
console.log(`🌐 主页: ${homepage}`);

// 1. 同步到 tauri.conf.json
try {
  const tauriConfigPath = join(rootDir, 'src-tauri/tauri.conf.json');
  const tauriConfig = JSON.parse(readFileSync(tauriConfigPath, 'utf-8'));

  tauriConfig.version = version;
  tauriConfig.mainBinaryName = name;

  writeFileSync(
    tauriConfigPath,
    JSON.stringify(tauriConfig, null, 2) + '\n',
    'utf-8'
  );
  console.log('✅ 已同步到 tauri.conf.json');
} catch (error) {
  console.error('❌ 同步 tauri.conf.json 失败:', error.message);
  process.exit(1);
}

// 2. 同步到 Cargo.toml
try {
  const cargoTomlPath = join(rootDir, 'src-tauri/Cargo.toml');
  let cargoToml = readFileSync(cargoTomlPath, 'utf-8');

  // 替换版本号
  cargoToml = cargoToml.replace(
    /^version = ".*"$/m,
    `version = "${version}"`
  );

  // 替换描述
  cargoToml = cargoToml.replace(
    /^description = ".*"$/m,
    `description = "${description}"`
  );

  writeFileSync(cargoTomlPath, cargoToml, 'utf-8');
  console.log('✅ 已同步到 Cargo.toml');
} catch (error) {
  console.error('❌ 同步 Cargo.toml 失败:', error.message);
  process.exit(1);
}

console.log('');
console.log('🎉 版本信息同步完成！');
console.log('');
