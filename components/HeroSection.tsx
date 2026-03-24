'use client';

import { Download, Monitor, Smartphone } from 'lucide-react';
import { OS, Release } from '@/lib/types';

interface HeroSectionProps {
  t: any;
  os: OS;
  releases: Release[];
  useChinaMirror: boolean;
}

export default function HeroSection({ t, os, releases, useChinaMirror }: HeroSectionProps) {
  const getPrimaryDownloadUrl = () => {
    if (!releases.length || !releases[0].assets) return '#download';
    const assets = releases[0].assets.filter((asset) => !asset.name.endsWith('.sig') && asset.name !== 'latest.json');

    let targetAsset = null;
    if (os === 'windows') {
      targetAsset = assets.find(a => a.name.endsWith('.exe'));
    } else if (os === 'macos') {
      targetAsset = assets.find(a => a.name.endsWith('.dmg')) || assets.find(a => a.name.toLowerCase().includes('mac'));
    } else if (os === 'linux') {
      targetAsset = assets.find(a => a.name.endsWith('.AppImage')) || assets.find(a => a.name.endsWith('.deb'));
    }

    if (targetAsset) {
      return useChinaMirror ? `https://ghfast.top/${targetAsset.browser_download_url}` : targetAsset.browser_download_url;
    }
    return '#download';
  };

  const getOsLabel = () => {
    if (os === 'windows') return t.dlWindows;
    if (os === 'macos') return t.dlMac;
    if (os === 'linux') return t.dlLinux;
    return t.dlUnknown;
  };

  return (
    <section className="relative z-10 pt-40 pb-20 px-4 sm:px-6 lg:px-8 max-w-7xl mx-auto flex flex-col items-center text-center">
      <div className="hero-element inline-flex items-center space-x-2 bg-[#C8102E]/10 border border-[#C8102E]/20 text-[#C8102E] dark:text-[#ff4d6d] px-4 py-1.5 rounded-full text-xs font-mono mb-8 backdrop-blur-sm">
        <span className="relative flex h-2 w-2">
          <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-[#C8102E] opacity-75"></span>
          <span className="relative inline-flex rounded-full h-2 w-2 bg-[#C8102E]"></span>
        </span>
        <span className="uppercase tracking-wider">{t.systemOnline}</span>
      </div>
      
      <h1 className="hero-element text-5xl md:text-7xl lg:text-8xl font-black tracking-tighter mb-6 text-slate-900 dark:text-transparent dark:bg-clip-text dark:bg-gradient-to-b dark:from-white dark:via-slate-200 dark:to-slate-500">
        {t.heroTitle1}<br />
        <span className="text-transparent bg-clip-text bg-gradient-to-r from-[#C8102E] via-[#FFC72C] to-[#C8102E]">{t.heroTitle2}</span>
      </h1>
      
      <p className="hero-element text-lg md:text-xl text-slate-600 dark:text-slate-400 max-w-2xl mb-10 font-light">
        {t.heroDesc}
      </p>
      
      <div className="hero-element flex flex-wrap justify-center items-center gap-4">
        {os === 'unknown' ? (
          <div className="flex items-center space-x-2 bg-black/5 dark:bg-white/5 text-slate-500 px-8 py-4 rounded-full font-bold cursor-not-allowed border border-black/10 dark:border-white/10 backdrop-blur-sm">
            <Monitor className="w-5 h-5" />
            <span>{getOsLabel()}</span>
          </div>
        ) : (
          <a
            href={getPrimaryDownloadUrl()}
            className="group relative flex items-center space-x-2 bg-[#00A651] hover:bg-[#008f45] text-white px-8 py-4 rounded-full font-bold transition-all hover:scale-105 shadow-[0_0_20px_rgba(0,166,81,0.3)]"
          >
            <div className="absolute inset-0 rounded-full bg-white blur-md opacity-0 group-hover:opacity-20 transition-opacity"></div>
            <Monitor className="w-5 h-5 relative z-10" />
            <span className="relative z-10">{getOsLabel()}</span>
          </a>
        )}
        <button
          onClick={() => document.getElementById('download')?.scrollIntoView({ behavior: 'smooth' })}
          className="flex items-center space-x-2 hover:cursor-pointer bg-black/5 hover:bg-black/10 dark:bg-white/10 dark:hover:bg-white/20 text-slate-800 dark:text-white px-8 py-4 rounded-full font-medium transition-all border border-black/10 dark:border-white/10 backdrop-blur-sm"
        >
          <Download className="w-5 h-5" />
          <span>{t.dlOther}</span>
        </button>
        <div className="flex items-center space-x-2 bg-black/5 dark:bg-white/5 text-slate-500 dark:text-slate-400 px-8 py-4 rounded-full font-medium cursor-not-allowed border border-black/10 dark:border-white/10 backdrop-blur-sm">
          <Smartphone className="w-5 h-5" />
          <span>{t.mobileWait}</span>
        </div>
      </div>
    </section>
  );
}
