import { fetch as tauriFetch } from '@tauri-apps/plugin-http'

const DB_NAME = 'image-cache-db'
const STORE_NAME = 'images'
let dbInstance: IDBDatabase | null = null

function initDB(): Promise<IDBDatabase> {
  if (dbInstance) return Promise.resolve(dbInstance)

  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, 1)

    request.onupgradeneeded = event => {
      const db = (event.target as IDBOpenDBRequest).result
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        db.createObjectStore(STORE_NAME)
      }
    }

    request.onsuccess = () => {
      dbInstance = request.result
      resolve(dbInstance)
    }

    request.onerror = () => reject(request.error)
  })
}

async function getFromDB(key: string): Promise<Blob | null> {
  const db = await initDB()
  return new Promise((resolve, reject) => {
    const transaction = db.transaction(STORE_NAME, 'readonly')
    const store = transaction.objectStore(STORE_NAME)
    const request = store.get(key)

    request.onsuccess = () => resolve(request.result || null)
    request.onerror = () => reject(request.error)
  })
}

async function saveToDB(key: string, blob: Blob): Promise<void> {
  const db = await initDB()
  return new Promise((resolve, reject) => {
    const transaction = db.transaction(STORE_NAME, 'readwrite')
    const store = transaction.objectStore(STORE_NAME)
    const request = store.put(blob, key)

    request.onsuccess = () => resolve()
    request.onerror = () => reject(request.error)
  })
}

/**
 * 获取并缓存图片，返回可用的本地 Blob URL
 * @param url 图片的原始 URL
 * @returns 缓存后的 Blob URL 或原始 URL（如果失败）
 */
export async function getCachedImageUrl(url: string): Promise<string> {
  if (
    !url ||
    url.startsWith('data:') ||
    url.startsWith('blob:') ||
    url.startsWith('/') ||
    url.startsWith('http://localhost')
  ) {
    return url
  }

  try {
    const cachedBlob = await getFromDB(url)
    if (cachedBlob) {
      return URL.createObjectURL(cachedBlob)
    }

    // 使用 Tauri 的 HTTP 客户端来避免 CORS 问题
    const response = await tauriFetch(url, {
      method: 'GET',
    })

    if (response.ok) {
      const arrayBuffer = await response.arrayBuffer()
      const mimeType = response.headers.get('content-type') || 'image/png'
      const blob = new Blob([arrayBuffer], { type: mimeType })

      // 保存到 IndexedDB
      await saveToDB(url, blob)

      return URL.createObjectURL(blob)
    }
  } catch (error) {
    console.warn('Image cache error for', url, error)
  }

  // 如果缓存失败，退回到原始 URL
  return url
}
