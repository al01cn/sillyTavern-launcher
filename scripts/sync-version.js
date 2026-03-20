import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const rootDir = path.resolve(__dirname, '..');
const packageJsonPath = path.join(rootDir, 'package.json');
const tauriConfPath = path.join(rootDir, 'src-tauri', 'tauri.conf.json');
const cargoTomlPath = path.join(rootDir, 'src-tauri', 'Cargo.toml');

try {
  // 1. 读取 package.json 的信息
  const packageJson = JSON.parse(fs.readFileSync(packageJsonPath, 'utf8'));
  const version = packageJson.version;
  const name = packageJson.name;
  const description = packageJson.description || '';

  if (!version || !name) {
    throw new Error('No version or name found in package.json');
  }

  console.log(`Syncing name: ${name}, description: ${description}, version: ${version} from package.json...`);

  // 2. 更新 tauri.conf.json
  const tauriConfStr = fs.readFileSync(tauriConfPath, 'utf8');
  const tauriConf = JSON.parse(tauriConfStr);
  
  let tauriConfUpdated = false;

  if (tauriConf.version !== version) {
    tauriConf.version = version;
    tauriConfUpdated = true;
    console.log(`✅ Updated tauri.conf.json version to ${version}`);
  }
  
  if (tauriConf.productName !== name) {
    tauriConf.productName = name;
    tauriConfUpdated = true;
    console.log(`✅ Updated tauri.conf.json productName to ${name}`);
  }

  if (tauriConfUpdated) {
    fs.writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + '\n', 'utf8');
  } else {
    console.log(`⚡ tauri.conf.json is already up to date`);
  }

  // 3. 更新 Cargo.toml
  let cargoTomlStr = fs.readFileSync(cargoTomlPath, 'utf8');
  
  // 匹配并替换 [package] 下的 version, name, description
  const versionRegex = /^(\s*version\s*=\s*")([^"]+)(")/m;
  const nameRegex = /^(\s*name\s*=\s*")([^"]+)(")/m;
  const descriptionRegex = /^(\s*description\s*=\s*")([^"]*)(")/m;
  
  let newCargoTomlStr = cargoTomlStr.replace(versionRegex, (match, p1, p2, p3) => {
      return `${p1}${version}${p3}`;
  });

  newCargoTomlStr = newCargoTomlStr.replace(nameRegex, (match, p1, p2, p3) => {
      // Cargo 包名通常全小写
      return `${p1}${name.toLowerCase()}${p3}`;
  });

  if (descriptionRegex.test(newCargoTomlStr)) {
    newCargoTomlStr = newCargoTomlStr.replace(descriptionRegex, (match, p1, p2, p3) => {
        return `${p1}${description}${p3}`;
    });
  } else {
    newCargoTomlStr = newCargoTomlStr.replace(versionRegex, (match) => {
        return `${match}\ndescription = "${description}"`;
    });
  }

  if (cargoTomlStr !== newCargoTomlStr) {
    fs.writeFileSync(cargoTomlPath, newCargoTomlStr, 'utf8');
    console.log(`✅ Updated Cargo.toml with new name/description/version`);
  } else {
    console.log(`⚡ Cargo.toml is already up to date`);
  }

} catch (error) {
  console.error('Error syncing version:', error);
  process.exit(1);
}
