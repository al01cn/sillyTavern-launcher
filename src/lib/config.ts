import pkg from '../../package.json'
import { PhOpenAiLogo, PhGoogleLogo } from '@phosphor-icons/vue'
import logo from '../assets/logo.svg'

export default {
    appName: '酒馆启动器GUI',
    appNameEn: "SillyTavern Launcher GUI",
    appVersion: pkg.version,
    appDescription: pkg.description,
    appDescriptionEn: pkg.descriptionEn,
    appHomepage: pkg.homepage,
    appIcon: logo,
    git: {
        github: "https://github.com/al01cn/sillyTavern-launcher",
        gitee: "https://gitee.com/al01/sillytavern-launcher"
    },
    tools: {
        "资源/工具": [
            {
                icon: "https://cdn.discordapp.com/icons/1134557553011998840/d419091a2a50009ddee0617ac43b0ead.png",
                name: '类脑',
                url: 'https://discord.gg/odysseia',
            },
            {
                icon: "https://cdn.discordapp.com/icons/1134557553011998840/d419091a2a50009ddee0617ac43b0ead.png",
                name: '类脑索引',
                url: 'https://odysseia-index.pages.dev/',
            },
            {
                icon: "https://sillytavern.wiki/favicon.ico",
                name: 'SillyTavern Wiki',
                url: 'https://sillytavern.wiki/',
            }
        ],
        "公益API": [
            {
                icon: "https://q1.qlogo.cn/g?b=qq&nk=790132463&s=100",
                name: '炫酷 API',
                url: 'https://new1.588686.xyz/register?aff=p4f6',
            },
            {
                icon: "https://i.ibb.co/pjd8dC8Y/7fcf0b96-69dd-4306-8d67-a1abde2ae423.png",
                name: 'Soul公益站',
                url: 'https://sunlea.de/',
            },
            {
                icon: "https://zipline.chat-linmou.online/u/a3nEt5.jpg",
                name: 'GuDuFree',
                url: 'https://gudufree.yeelam.site/register?aff=YVo4',
            }
        ],
        "半公益API": [
            {
                icon: "https://free.supxh.xin/favicon.ico",
                name: '肖恩AI',
                url: 'https://free.supxh.xin/',
            },
        ],
        "付费API": [
            {
                icon: "https://www.wamwuai.com/logo.png",
                name: '万物科技',
                url: 'https://www.wamwuai.com/register?aff=IzGc',
            },
            {
                icon: "https://juziapi.xin/logo.png",
                name: '橘子API',
                url: 'https://juziapi.xin/register?aff=5pPk',
            },
            {
                icon: "https://lsky.zhongzhuan.chat/i/2026/03/14/69b552d289fd8.png",
                name: '镜芯AI',
                url: 'https://ai.wer.plus/register?aff=jIR0',
            },
            {
                icon: "https://apiyi.com/images/favicon-64x64.png",
                name: 'API易',
                url: 'https://api.apiyi.com/register/?aff_code=h5pY',
            },
            {
                icon: "https://ppio.com/favicon.ico",
                name: 'PPIO 派欧云',
                url: 'https://ppio.com/user/register?invited_by=OOC1DK',
            },
            {
                icon: "https://megallm.io/_next/image?url=%2Fmegallm-logo-dark.png&w=96&q=75",
                name: 'Megallm',
                url: 'https://megallm.io/',
            }
        ],
        "AI模型": [
            {
                icon: "https://www.deepseek.com/favicon.ico",
                name: 'DeepSeek',
                url: 'https://www.deepseek.com/',
            },
            {
                icon: "https://minimaxi.com/favicon.ico",
                name: 'Minimaxi CN',
                url: 'https://minimaxi.com/',
            },
            {
                icon: "https://www.minimax.io/favicon.ico",
                name: 'Minimax',
                url: 'https://www.minimax.io/',
            },
            {
                icon: "https://www.gstatic.com/lamda/images/gemini_sparkle_aurora_33f86dc0c0257da337c63.svg",
                defaultIcon: PhGoogleLogo,
                name: 'Gemini',
                url: 'https://gemini.google.com/',
            },
            {
                icon: "https://chatgpt.com/favicon.ico",
                defaultIcon: PhOpenAiLogo,
                name: 'ChatGPT',
                url: 'https://chatgpt.com/',
            },
            {
                icon: "https://assets-proxy.anthropic.com/claude-ai/v2/assets/v1/cd02a42d9-Vq_H3mgS.svg",
                name: 'Claude',
                url: 'https://claude.ai/',
            }
        ]
    }
}