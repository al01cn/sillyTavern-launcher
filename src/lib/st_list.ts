// SillyTavern 版本列表回退数据
// 当 GitHub API 请求失败时使用此数据

export interface ReleaseAsset {
    name: string;
    browser_download_url: string;
}

export interface Release {
    id: number;
    tag_name: string;
    name: string;
    body: string;
    created_at: string;
    published_at: string;
    zipball_url: string;
    assets: ReleaseAsset[];
}

// 回退版本列表（包含最近的稳定版本）
export const fallbackReleases: Release[] = [
    {
        id: 1,
        tag_name: "1.16.0",
        name: "1.16.0",
        body: "SillyTavern 1.16.0 Release",
        created_at: "2025-01-15T00:00:00Z",
        published_at: "2025-01-15T00:00:00Z",
        zipball_url: "https://github.com/SillyTavern/SillyTavern/archive/refs/tags/1.16.0.zip",
        assets: []
    },
    {
        id: 2,
        tag_name: "1.15.0",
        name: "1.15.0",
        body: "SillyTavern 1.15.0 Release",
        created_at: "2024-12-01T00:00:00Z",
        published_at: "2024-12-01T00:00:00Z",
        zipball_url: "https://github.com/SillyTavern/SillyTavern/archive/refs/tags/1.15.0.zip",
        assets: []
    },
    {
        id: 3,
        tag_name: "1.14.0",
        name: "1.14.0",
        body: "SillyTavern 1.14.0 Release",
        created_at: "2024-10-15T00:00:00Z",
        published_at: "2024-10-15T00:00:00Z",
        zipball_url: "https://github.com/SillyTavern/SillyTavern/archive/refs/tags/1.14.0.zip",
        assets: []
    },
    {
        id: 4,
        tag_name: "1.13.0",
        name: "1.13.0",
        body: "SillyTavern 1.13.0 Release",
        created_at: "2024-09-01T00:00:00Z",
        published_at: "2024-09-01T00:00:00Z",
        zipball_url: "https://github.com/SillyTavern/SillyTavern/archive/refs/tags/1.13.0.zip",
        assets: []
    },
    {
        id: 5,
        tag_name: "1.12.6",
        name: "1.12.6",
        body: "SillyTavern 1.12.6 Release",
        created_at: "2024-07-15T00:00:00Z",
        published_at: "2024-07-15T00:00:00Z",
        zipball_url: "https://github.com/SillyTavern/SillyTavern/archive/refs/tags/1.12.6.zip",
        assets: []
    }
];
