// SillyTavern 版本列表回退数据
// 当 GitHub API 请求失败时使用此数据

export interface ReleaseAsset {
  name: string
  browser_download_url: string
}

export interface Release {
  id: number
  tag_name: string
  name: string
  body: string
  created_at: string
  updated_at: string
  published_at: string
  zipball_url: string
  assets: ReleaseAsset[]
}

// 回退版本列表（包含最近的稳定版本）
export const fallbackReleases: Release[] = [
  {
    id: 1,
    tag_name: '1.16.0',
    name: '1.16.0',
    body: 'SillyTavern 1.16.0 Release',
    created_at: '2026-02-14T15:46:49Z',
    updated_at: '2026-03-25T19:41:34Z',
    published_at: '2026-02-14T15:47:53Z',
    zipball_url: 'https://github.com/SillyTavern/SillyTavern/archive/refs/tags/1.16.0.zip',
    assets: [],
  },
  {
    id: 2,
    tag_name: '1.15.0',
    name: '1.15.0',
    body: 'SillyTavern 1.15.0 Release',
    created_at: '2025-12-28T16:28:51Z',
    updated_at: '2026-03-25T19:41:29Z',
    published_at: '2025-12-28T16:29:48Z',
    zipball_url: 'https://github.com/SillyTavern/SillyTavern/archive/refs/tags/1.15.0.zip',
    assets: [],
  },
  {
    id: 3,
    tag_name: '1.14.0',
    name: '1.14.0',
    body: 'SillyTavern 1.14.0 Release',
    created_at: '2025-11-22T15:27:11Z',
    updated_at: '2026-03-25T19:41:26Z',
    published_at: '2025-11-22T15:28:06Z',
    zipball_url: 'https://github.com/SillyTavern/SillyTavern/archive/refs/tags/1.14.0.zip',
    assets: [],
  },
  {
    id: 4,
    tag_name: '1.13.5',
    name: '1.13.5',
    body: 'SillyTavern 1.13.5 Release',
    created_at: '2025-10-16T17:31:22Z',
    updated_at: '2026-03-25T19:41:22Z',
    published_at: '2025-10-16T17:32:15Z',
    zipball_url: 'https://github.com/SillyTavern/SillyTavern/archive/refs/tags/1.13.5.zip',
    assets: [],
  },
]
