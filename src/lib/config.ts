import pkg from '../../package.json'
import { PhOpenAiLogo, PhGoogleLogo } from '@phosphor-icons/vue'
import logo from '../assets/logo.svg'

export default {
  appName: '酒馆启动器GUI',
  appNameEn: 'SillyTavern Launcher GUI',
  appVersion: pkg.version,
  appDescription: pkg.description,
  appDescriptionEn: pkg.descriptionEn,
  appHomepage: pkg.homepage,
  appIcon: logo,
  git: {
    github: 'https://github.com/al01cn/sillyTavern-launcher',
    gitee: 'https://gitee.com/al01/sillytavern-launcher',
  },
  tools: {
    '资源/工具': [
      {
        icon: 'https://cdn.discordapp.com/icons/1134557553011998840/d419091a2a50009ddee0617ac43b0ead.png',
        name: '类脑',
        url: 'https://discord.gg/odysseia',
      },
      {
        icon: 'https://cdn.discordapp.com/icons/1134557553011998840/d419091a2a50009ddee0617ac43b0ead.png',
        name: '类脑索引',
        url: 'https://odysseia-index.pages.dev/',
      },
      {
        icon: 'https://sillytavern.wiki/favicon.ico',
        name: 'SillyTavern Wiki',
        url: 'https://docs.sillytavern.app/',
      },
      {
        icon: 'https://sillytavern.wiki/favicon.ico',
        name: 'SillyTavern 中文文档',
        url: 'https://sillytavern.wiki/',
      },
    ],
    公益API: [
      {
        icon: 'https://q1.qlogo.cn/g?b=qq&nk=790132463&s=100',
        name: '炫酷 API',
        url: 'https://new1.588686.xyz/register?aff=p4f6',
      },
      {
        icon: 'https://zipline.chat-linmou.online/u/a3nEt5.jpg',
        name: 'GuDuFree',
        url: 'https://gudufree.yeelam.site/register?aff=YVo4',
      },
    ],
    半公益API: [
      {
        icon: 'https://free.supxh.xin/favicon.ico',
        name: '肖恩AI',
        url: 'https://free.supxh.xin/',
      },
    ],
    付费API: [
      {
        icon: 'https://www.wamwuai.com/logo.png',
        name: '万物科技',
        url: 'https://www.wamwuai.com/register?aff=IzGc',
      },
      {
        icon: 'https://juziapi.xin/logo.png',
        name: '橘子API',
        url: 'https://juziapi.xin/register?aff=5pPk',
      },
      {
        icon: 'https://lsky.zhongzhuan.chat/i/2026/03/14/69b552d289fd8.png',
        name: '镜芯AI',
        url: 'https://ai.wer.plus/register?aff=jIR0',
      },
      {
        icon: 'https://apiyi.com/images/favicon-64x64.png',
        name: 'API易',
        url: 'https://api.apiyi.com/register/?aff_code=h5pY',
      },
      {
        icon: 'https://ppio.com/favicon.ico',
        name: 'PPIO 派欧云',
        url: 'https://ppio.com/user/register?invited_by=OOC1DK',
      },
      {
        icon: 'https://megallm.io/_next/image?url=%2Fmegallm-logo-dark.png&w=96&q=75',
        name: 'Megallm',
        url: 'https://megallm.io/',
      },
    ],
    AI模型: [
      {
        icon: 'https://www.deepseek.com/favicon.ico',
        name: 'DeepSeek',
        url: 'https://www.deepseek.com/',
      },
      {
        icon: 'https://minimaxi.com/favicon.ico',
        name: 'Minimaxi CN',
        url: 'https://minimaxi.com/',
      },
      {
        icon: 'https://www.minimax.io/favicon.ico',
        name: 'Minimax',
        url: 'https://www.minimax.io/',
      },
      {
        icon: 'https://www.gstatic.com/lamda/images/gemini_sparkle_aurora_33f86dc0c0257da337c63.svg',
        defaultIcon: PhGoogleLogo,
        name: 'Gemini',
        url: 'https://gemini.google.com/',
      },
      {
        icon: 'https://chatgpt.com/favicon.ico',
        defaultIcon: PhOpenAiLogo,
        name: 'ChatGPT',
        url: 'https://chatgpt.com/',
      },
      {
        icon: 'https://assets-proxy.anthropic.com/claude-ai/v2/assets/v1/cd02a42d9-Vq_H3mgS.svg',
        name: 'Claude',
        url: 'https://claude.ai/',
      },
    ],
  },
  ca: {
    categories: [
      {
        name: '框架 / Frameworks',
        items: [
          { name: 'Tauri', version: '2', url: 'https://tauri.app/', key: 'tauri' },
          { name: 'Vue', version: '3.5', url: 'https://vuejs.org/', key: 'vue' },
          { name: 'Rust', version: '1.75+', url: 'https://www.rust-lang.org/', key: 'rust' },
        ],
      },
      {
        name: '前端依赖 / Frontend',
        items: [
          { name: 'Tailwind CSS', version: '4.2', url: 'https://tailwindcss.com/', key: 'tailwind' },
          { name: 'vue-i18n', version: '11', url: 'https://vue-i18n.intlify.dev/', key: 'vueI18n' },
          { name: 'vue-router', version: '5', url: 'https://router.vuejs.org/', key: 'vueRouter' },
          { name: 'Phosphor Icons', version: '2.2', url: 'https://phosphoricons.com/', key: 'phosphorIcons' },
          { name: 'Lucide Vue', version: '0.577', url: 'https://lucide.dev/', key: 'lucide' },
          { name: 'QRCode', version: '1.5', url: 'https://github.com/soldair/node-qrcode', key: 'qrcode' },
          { name: 'Vue Sonner', version: '2', url: 'https://github.com/AntonyAnu/sonner-vue', key: 'vueSonner' },
          { name: 'DaisyUI', version: '5', url: 'https://daisyui.com/', key: 'daisyui' },
        ],
      },
      {
        name: '后端依赖 / Backend',
        items: [
          { name: 'Tokio', version: '1', url: 'https://tokio.rs/', key: 'tokio' },
          { name: 'Reqwest', version: '0.12', url: 'https://docs.rs/reqwest/0.12/reqwest/', key: 'reqwest' },
          { name: 'Serde', version: '1', url: 'https://serde.rs/', key: 'serde' },
          { name: 'Zip', version: '0.6', url: 'https://github.com/zip-rs/zip', key: 'zip' },
          { name: 'Walkdir', version: '2.5', url: 'https://github.com/BurntSushi/walkdir', key: 'walkdir' },
          { name: 'Jwalk', version: '0.8', url: 'https://github.com/Byron/jwalk', key: 'jwalk' },
          { name: 'Sevenz', version: '0.6', url: 'https://github.com/erthink/7z', key: 'sevenz' },
          {
            name: 'Headless Chrome',
            version: '1',
            url: 'https://github.com/ChromeDevTools/headless_chrome',
            key: 'headlessChrome',
          },
          { name: 'Winreg', version: '0.52', url: 'https://github.com/gentoo90/winreg', key: 'winreg' },
        ],
      },
      {
        name: '开发工具 / DevTools',
        items: [
          { name: 'Vite', version: '6', url: 'https://vite.dev/', key: 'vite' },
          { name: 'TypeScript', version: '6', url: 'https://www.typescriptlang.org/', key: 'typescript' },
          { name: 'ESLint', version: '10', url: 'https://eslint.org/', key: 'eslint' },
          { name: 'Prettier', version: '3.8', url: 'https://prettier.io/', key: 'prettier' },
        ],
      },
      {
        name: '特别感谢 / Special Thanks',
        items: [
          { name: 'SillyTavern', version: '1.1x.x', url: 'https://sillytavern.app/', key: 'sillytavern' },
          {
            name: 'SillyTavern 社区',
            version: '',
            url: 'https://github.com/SillyTavern/SillyTavern',
            key: 'sillytavernCommunity',
          },
          {
            name: 'Github Proxy',
            version: '',
            url: 'https://github.akams.cn/',
            key: 'githubProxy',
          },
          {
            name: 'Github Proxy - ghfast',
            version: '',
            url: 'https://ghfast.top/',
            key: 'ghfast',
          },
        ],
      },
    ],
  },
}
