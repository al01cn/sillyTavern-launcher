'use client';

import { useState, useEffect } from 'react';

/**
 * 主题切换 Hook
 * 管理深色/浅色主题状态，并同步到 localStorage 和 DOM
 */
export function useTheme() {
  const [isDark, setIsDark] = useState(true);

  useEffect(() => {
    // 仅在客户端执行
    if (typeof window !== 'undefined') {
      const storedTheme = localStorage.getItem('theme');
      const isSystemDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      
      // 如果有存储的主题设置，使用存储的；否则跟随系统
      if (storedTheme) {
        setIsDark(storedTheme === 'dark');
      } else {
        setIsDark(isSystemDark);
      }
    }
  }, []);

  useEffect(() => {
    // 当主题状态变化时，更新 DOM 和 localStorage
    if (isDark) {
      document.documentElement.classList.add('dark');
      localStorage.setItem('theme', 'dark');
    } else {
      document.documentElement.classList.remove('dark');
      localStorage.setItem('theme', 'light');
    }
  }, [isDark]);

  const toggleTheme = () => setIsDark(prev => !prev);

  return { isDark, toggleTheme };
}
