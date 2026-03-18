import { ref, watch, Ref, UnwrapRef } from 'vue'

export class localCache {
  /**
   * 获取一个响应式缓存
   * @param key localStorage key
   * @param defaultValue 默认值
   */
  static get<T>(key: string, defaultValue: T | null = null): Ref<UnwrapRef<T> | null> {
    const stored = localStorage.getItem(key)
    const initialValue = stored !== null ? (JSON.parse(stored) as T) : defaultValue

    const data: Ref<UnwrapRef<T> | null> = ref(initialValue as any)

    // 监听 ref 变化，自动同步 localStorage
    watch(
      data,
      (newVal) => {
        if (newVal === null || newVal === undefined) {
          localStorage.removeItem(key)
        } else {
          localStorage.setItem(key, JSON.stringify(newVal))
        }
      },
      { deep: true }
    )

    return data
  }

  static set<T>(key: string, value: T) {
    localStorage.setItem(key, JSON.stringify(value))
  }

  static remove(key: string) {
    localStorage.removeItem(key)
  }
}

// ⚡ 关键：在类外部创建一个缓存池，存储已经生成的 ref 实例
const sessionRefStore = new Map<string, Ref<any>>();

export class SessionCache {
  static get<T>(key: string, defaultValue: T | null = null): Ref<UnwrapRef<T>> {
    // 如果已经存在该 key 的 ref，直接返回，不再重新创建
    if (sessionRefStore.has(key)) {
      return sessionRefStore.get(key)!;
    }

    const stored = sessionStorage.getItem(key)
    const initialValue = stored !== null ? (JSON.parse(stored) as T) : defaultValue
    const data = ref(initialValue as any)

    watch(
      data,
      (newVal) => {
        if (newVal === null || newVal === undefined) {
          sessionStorage.removeItem(key)
        } else {
          sessionStorage.setItem(key, JSON.stringify(newVal))
        }
      },
      { deep: true }
    )

    // 存入缓存池
    sessionRefStore.set(key, data);
    return data;
  }

  // ⚡ 修改 set 方法，使其能同步现有的 ref
  static set<T>(key: string, value: T) {
    sessionStorage.setItem(key, JSON.stringify(value));
    if (sessionRefStore.has(key)) {
      sessionRefStore.get(key)!.value = value;
    }
  }

  static remove(key: string) {
    sessionStorage.removeItem(key)
    if (sessionRefStore.has(key)) {
      sessionRefStore.get(key)!.value = null;
    }
  }
}

export default SessionCache