import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { fallbackReleases, Release } from './st_list';

const releases = ref<Release[]>([]);
const lastFetchTime = ref<number>(0);
const loading = ref(false);

// 配置常量
const CACHE_KEY = 'sillytavern_releases_cache';
const LAST_FETCH_KEY = 'sillytavern_releases_last_fetch';
const THREE_DAYS_MS = 3 * 24 * 60 * 60 * 1000;

/**
 * 初始化发布列表管理
 * 优先从本地缓存加载以实现秒开
 */
export function initReleases() {
    const cachedReleases = localStorage.getItem(CACHE_KEY);
    const cachedTime = localStorage.getItem(LAST_FETCH_KEY);
    
    if (cachedReleases) {
        try {
            const parsed = JSON.parse(cachedReleases);
            if (Array.isArray(parsed) && parsed.length > 0) {
                releases.value = parsed;
            }
        } catch (e) {
            console.error('Failed to parse releases cache:', e);
        }
    }
    
    if (cachedTime) {
        lastFetchTime.value = Number(cachedTime);
    }
    
    // 如果没有缓存或缓存过期，自动触发后台更新
    const now = Date.now();
    if (!releases.value.length || (now - lastFetchTime.value > THREE_DAYS_MS)) {
        fetchReleases().catch(console.error);
    }
}

/**
 * 后台获取最新的发布列表并更新缓存
 * @param force 是否强制更新
 */
export async function fetchReleases(force = false) {
    if (loading.value && !force) return releases.value;
    
    const now = Date.now();
    // 强制更新或缓存过期时才真正发起请求
    if (!force && releases.value.length > 0 && (now - lastFetchTime.value < THREE_DAYS_MS)) {
        return releases.value;
    }
    
    loading.value = true;
    try {
        const fetchedReleases = await invoke<Release[]>('fetch_sillytavern_releases');
        if (Array.isArray(fetchedReleases) && fetchedReleases.length > 0) {
            releases.value = fetchedReleases;
            localStorage.setItem(CACHE_KEY, JSON.stringify(fetchedReleases));
            localStorage.setItem(LAST_FETCH_KEY, now.toString());
            lastFetchTime.value = now;
        }
    } catch (error) {
        console.error('Failed to fetch releases:', error);
        // 如果环境是空且请求失败，使用回退数据
        if (releases.value.length === 0) {
            releases.value = fallbackReleases;
            localStorage.setItem(CACHE_KEY, JSON.stringify(fallbackReleases));
        }
    } finally {
        loading.value = false;
    }
    
    return releases.value;
}

/**
 * 钩子导出：获取当前发布列表状态
 */
export function useReleases() {
    const latestVersion = computed(() => releases.value.length > 0 ? releases.value[0].tag_name : '');
    
    return {
        releases,
        latestVersion,
        loading,
        lastFetchTime,
        fetchReleases,
        initReleases
    };
}
