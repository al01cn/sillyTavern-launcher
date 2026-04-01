import js from '@eslint/js'
import ts from 'typescript-eslint'
import vue from 'eslint-plugin-vue'
import vueParser from 'vue-eslint-parser'
import prettierConfig from 'eslint-config-prettier'
import prettierPlugin from 'eslint-plugin-prettier'
import globals from 'globals'

export default ts.config(
  // 1. 基础 JS 推荐配置
  js.configs.recommended,

  // 2. TypeScript 推荐配置
  ...ts.configs.recommended,

  // 3. Vue 3 配置
  ...vue.configs['flat/recommended'],

  {
    // 指定处理的文件范围
    files: ['**/*.ts', '**/*.vue'],
    languageOptions: {
      globals: {
        ...globals.browser, // 注入浏览器全局变量 (window, document 等)
        ...globals.node, // 如果是 Node 环境也需要
        ...globals.es2021,
      },
      parser: vueParser, // 使用 vue 解析器
      parserOptions: {
        parser: ts.parser, // 在 Vue 内部使用 TS 解析器
        sourceType: 'module',
        ecmaVersion: 'latest',
      },
    },
    plugins: {
      prettier: prettierPlugin,
    },
    rules: {
      // 合并 Prettier 规则
      ...prettierConfig.rules,

      // 1. 允许声明但未使用的变量以下划线开头 (如 _index)
      '@typescript-eslint/no-unused-vars': [
        'warn',
        {
          argsIgnorePattern: '^_',
          varsIgnorePattern: '^_',
          caughtErrors: 'all',
          caughtErrorsIgnorePattern: '^_',
        },
      ],

      // 自定义规则
      'vue/multi-word-component-names': 'off',
      // v-html 在聊天/角色卡渲染场景下是必要的，关闭 XSS 误报
      'vue/no-v-html': 'off',
      // 2. 允许空的 catch 块（有时我们确实只想静默失败）
      'no-empty': ['error', { allowEmptyCatch: true }],

      // 3. 针对角色卡这种复杂对象，暂时允许 any
      '@typescript-eslint/no-explicit-any': 'off',

      // 4. 强制 Prettier 格式
      'prettier/prettier': 'error',

      '@typescript-eslint/no-unused-expressions': 'off',
      '@typescript-eslint/no-this-alias': 'off', // 顺便屏蔽那个 'this' 别名的报错
    },
  },

  // 4. 忽略文件 (替代 .eslintignore)
  {
    ignores: [
      '**/dist/**',
      '**/node_modules/**',
      '**/src-tauri/target/**',
      '**/src-tauri/gen/**',
      '**/*.min.js', // 忽略所有压缩文件
      'public/vendor/**', // 如果你有存放第三方库的目录，也加上
      'scripts/**',
      'data/**',
    ],
  },
)
