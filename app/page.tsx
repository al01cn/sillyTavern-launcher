'use client';

import { useEffect, useState } from 'react';
import { gsap } from 'gsap';
import { ScrollTrigger } from 'gsap/ScrollTrigger';
import { Navigation, HeroSection, FeaturesSection, DownloadSection, Footer, LanguageSwitcher } from '@/components';
import { translations, Release, OS, Lang, detectOS, shouldUseChinaMirror } from '@/lib/types';
import { extractLocaleFromPath, type Locale } from '@/lib/i18n-config';
import { useTheme } from '@/hooks/use-theme';

gsap.registerPlugin(ScrollTrigger);

export default function Home() {
  const [releases, setReleases] = useState<Release[]>([]); // 移动端
  const [pcReleases, setPcReleases] = useState<Release[]>([]); // PC 端
  const [loading, setLoading] = useState(true);
  const [pcLoading, setPcLoading] = useState(true);
  const [useChinaMirror, setUseChinaMirror] = useState(false);
  const [os, setOs] = useState<OS>('unknown');
  const [lang, setLang] = useState<Lang>('zh');
  const [currentLocale, setCurrentLocale] = useState<Locale>('zh');
  const { isDark, toggleTheme } = useTheme();

  const t = translations[lang];

  // 获取当前路径的语言
  useEffect(() => {
    if (typeof window !== 'undefined') {
      const locale = extractLocaleFromPath(window.location.pathname);
      setCurrentLocale(locale);
      setLang(locale);
    }
  }, []);

  useEffect(() => {
    // Auto-detect China region based on timezone
    setUseChinaMirror(shouldUseChinaMirror());

    // Auto-detect OS
    setOs(detectOS());

    // 获取移动端 releases
    const MOBILE_CACHE_KEY = 'st_launcher_mobile_releases_cache';
    const MOBILE_CACHE_TIME_KEY = 'st_launcher_mobile_releases_time';
    const CACHE_DURATION = 1000 * 60 * 60; // 1 hour

    const mobileCachedData = localStorage.getItem(MOBILE_CACHE_KEY);
    const mobileCachedTime = localStorage.getItem(MOBILE_CACHE_TIME_KEY);

    if (mobileCachedData && mobileCachedTime && (Date.now() - parseInt(mobileCachedTime) < CACHE_DURATION)) {
      setReleases(JSON.parse(mobileCachedData));
      setLoading(false);
    } else {
      fetch('https://api.github.com/repos/al01cn/sillytavern-launcher-mobile/releases')
        .then((res) => res.json())
        .then((data) => {
          if (Array.isArray(data)) {
            setReleases(data);
            localStorage.setItem(MOBILE_CACHE_KEY, JSON.stringify(data));
            localStorage.setItem(MOBILE_CACHE_TIME_KEY, Date.now().toString());
          } else if (data.message && data.message.includes('rate limit')) {
            if (mobileCachedData) setReleases(JSON.parse(mobileCachedData));
          }
          setLoading(false);
        })
        .catch((err) => {
          console.error('Failed to fetch mobile releases:', err);
          if (mobileCachedData) setReleases(JSON.parse(mobileCachedData));
          setLoading(false);
        });
    }

    // 获取 PC 端 releases
    const PC_CACHE_KEY = 'st_launcher_pc_releases_cache';
    const PC_CACHE_TIME_KEY = 'st_launcher_pc_releases_time';

    const pcCachedData = localStorage.getItem(PC_CACHE_KEY);
    const pcCachedTime = localStorage.getItem(PC_CACHE_TIME_KEY);

    if (pcCachedData && pcCachedTime && (Date.now() - parseInt(pcCachedTime) < CACHE_DURATION)) {
      setPcReleases(JSON.parse(pcCachedData));
      setPcLoading(false);
    } else {
      fetch('https://api.github.com/repos/al01cn/sillyTavern-launcher/releases')
        .then((res) => res.json())
        .then((data) => {
          if (Array.isArray(data)) {
            setPcReleases(data);
            localStorage.setItem(PC_CACHE_KEY, JSON.stringify(data));
            localStorage.setItem(PC_CACHE_TIME_KEY, Date.now().toString());
          } else if (data.message && data.message.includes('rate limit')) {
            if (pcCachedData) setPcReleases(JSON.parse(pcCachedData));
          }
          setPcLoading(false);
        })
        .catch((err) => {
          console.error('Failed to fetch PC releases:', err);
          if (pcCachedData) setPcReleases(JSON.parse(pcCachedData));
          setPcLoading(false);
        });
    }
  }, []);

  useEffect(() => {
    const ctx = gsap.context(() => {
      // Hero Animations
      gsap.fromTo(
        '.hero-element',
        { y: 40, opacity: 0, filter: 'blur(10px)' },
        { y: 0, opacity: 1, filter: 'blur(0px)', duration: 1, stagger: 0.15, ease: 'power3.out' }
      );

      // Features Animations
      gsap.fromTo(
        '.feature-card',
        { y: 50, opacity: 0, scale: 0.95 },
        {
          y: 0,
          opacity: 1,
          scale: 1,
          duration: 0.8,
          stagger: 0.1,
          ease: 'power2.out',
          scrollTrigger: {
            trigger: '.features-section',
            start: 'top 80%',
          },
        }
      );

      // Download Animations
      gsap.fromTo(
        '.download-section',
        { y: 30, opacity: 0 },
        {
          y: 0,
          opacity: 1,
          duration: 0.8,
          ease: 'power2.out',
          scrollTrigger: {
            trigger: '#download',
            start: 'top 85%',
          },
        }
      );
    });

    return () => ctx.revert();
  }, []);

  return (
    <div className="min-h-screen bg-slate-50 dark:bg-[#050505] text-slate-800 dark:text-slate-300 font-sans selection:bg-[#C8102E]/30 relative overflow-hidden transition-colors duration-300">
      {/* Tech Background Grid & Glow */}
      <div className="absolute inset-0 z-0 pointer-events-none">
        <div className="absolute inset-0 bg-[linear-gradient(to_right,#0000000a_1px,transparent_1px),linear-gradient(to_bottom,#0000000a_1px,transparent_1px)] dark:bg-[linear-gradient(to_right,#ffffff0a_1px,transparent_1px),dark:linear-gradient(to_bottom,#ffffff0a_1px,transparent_1px)] bg-[size:24px_24px] [mask-image:radial-gradient(ellipse_60%_50%_at_50%_0%,#000_70%,transparent_100%)]"></div>
        <div className="absolute top-0 left-1/2 -translate-x-1/2 w-[800px] h-[400px] opacity-20 blur-[120px] bg-gradient-to-b from-[#C8102E] via-[#FFC72C] to-transparent rounded-full"></div>
      </div>

      <Navigation 
        t={t} 
        lang={lang} 
        isDark={isDark} 
        onToggleTheme={toggleTheme} 
      />
      
      <HeroSection 
        t={t} 
        os={os} 
        releases={releases}
        pcReleases={pcReleases}
        useChinaMirror={useChinaMirror} 
      />
      
      <div className="features-section">
        <FeaturesSection t={t} />
      </div>
      
      <DownloadSection 
        t={t} 
        releases={releases} 
        loading={loading} 
        useChinaMirror={useChinaMirror} 
        onToggleChinaMirror={() => setUseChinaMirror(!useChinaMirror)}
        pcReleases={pcReleases}
        pcLoading={pcLoading}
      />
      
      <Footer t={t} lang={lang} />
    </div>
  );
}
