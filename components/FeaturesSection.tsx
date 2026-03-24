'use client';

import { Globe, Zap, Settings } from 'lucide-react';

interface FeaturesSectionProps {
  t: any;
}

export default function FeaturesSection({ t }: FeaturesSectionProps) {
  return (
    <section className="features-section relative z-10 py-24 border-y border-black/5 dark:border-white/5 bg-white/50 dark:bg-black/50 backdrop-blur-xl transition-colors duration-300">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="text-center mb-16">
          <h2 className="text-3xl md:text-4xl font-bold mb-4 text-slate-900 dark:text-white tracking-tight">{t.featuresTitle}</h2>
          <p className="text-slate-500 dark:text-slate-400 font-mono text-sm uppercase tracking-widest">{t.featuresSub}</p>
        </div>
        
        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {[
            {
              icon: <Globe className="w-6 h-6 text-[#C8102E]" />,
              title: t.feat1Title,
              desc: t.feat1Desc,
              glow: 'group-hover:shadow-[0_0_30px_rgba(200,16,46,0.15)]'
            },
            {
              icon: <Zap className="w-6 h-6 text-[#FFC72C]" />,
              title: t.feat2Title,
              desc: t.feat2Desc,
              glow: 'group-hover:shadow-[0_0_30px_rgba(255,199,44,0.15)]'
            },
            {
              icon: <Settings className="w-6 h-6 text-[#00A651]" />,
              title: t.feat3Title,
              desc: t.feat3Desc,
              glow: 'group-hover:shadow-[0_0_30px_rgba(0,166,81,0.15)]'
            }
          ].map((feature, idx) => (
            <div key={idx} className={`group feature-card bg-black/[0.02] dark:bg-white/[0.02] p-8 rounded-3xl border border-black/5 dark:border-white/5 transition-all duration-300 hover:bg-black/[0.04] dark:hover:bg-white/[0.04] hover:-translate-y-1 ${feature.glow}`}>
              <div className="w-12 h-12 bg-white dark:bg-white/5 rounded-2xl flex items-center justify-center mb-6 border border-black/10 dark:border-white/10 shadow-sm dark:shadow-none">
                {feature.icon}
              </div>
              <h3 className="text-xl font-bold mb-3 text-slate-900 dark:text-white">{feature.title}</h3>
              <p className="text-slate-600 dark:text-slate-400 leading-relaxed text-sm">
                {feature.desc}
              </p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
