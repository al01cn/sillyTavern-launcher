import { NextResponse } from 'next/server';
import type { NextRequest } from 'next/server';

// 支持的语言列表
const locales = ['zh', 'en'];
const defaultLocale: string = 'zh';

// 不需要语言前缀的路径（静态资源、API 等）
const ignoredPaths = ['/api', '/_next', '/static', '/favicon.ico'];

/**
 * 从请求中获取语言偏好
 */
function getLocaleFromRequest(request: NextRequest): string {
  // 1. 首先检查 URL 路径中的语言前缀
  const pathname = request.nextUrl.pathname;
  const pathnameParts = pathname.split('/');
  
  if (pathnameParts[1] && locales.includes(pathnameParts[1])) {
    return pathnameParts[1];
  }
  
  // 2. 检查 Accept-Language header
  const acceptLanguage = request.headers.get('accept-language');
  if (acceptLanguage) {
    const preferredLocale = acceptLanguage.split(',')[0].split('-')[0];
    if (locales.includes(preferredLocale)) {
      return preferredLocale;
    }
  }
  
  // 3. 返回默认语言
  return defaultLocale;
}

/**
 * 中间件函数 - 处理国际化路由
 */
export function middleware(request: NextRequest) {
  const { pathname } = request.nextUrl;
  
  // 忽略不需要语言前缀的路径
  if (ignoredPaths.some(path => pathname.startsWith(path))) {
    return NextResponse.next();
  }
  
  // 检查路径是否已有语言前缀
  const hasLocalePrefix = locales.some(loc => 
    pathname === `/${loc}` || pathname.startsWith(`/${loc}/`)
  );
  
  // 如果已经有语言前缀，直接放行
  if (hasLocalePrefix) {
    return NextResponse.next();
  }
  
  // 没有语言前缀，需要重定向
  // 获取用户的语言偏好
  const locale = getLocaleFromRequest(request);
  
  // 如果是默认语言（中文），不重定向，直接放行
  if (locale === defaultLocale) {
    return NextResponse.next();
  }
  
  // 非默认语言，添加语言前缀并重定向
  const newPathname = `/${locale}${pathname}`;
  const newUrl = new URL(newPathname, request.url);
  
  // 使用 301 永久重定向，有利于 SEO
  return NextResponse.redirect(newUrl, 301);
}

/**
 * 配置中间件匹配的路由
 */
export const config = {
  matcher: [
    /*
     * 匹配所有路径，除了：
     * - api (API routes)
     * - _next/static (静态文件)
     * - _next/image (图片优化)
     * - favicon.ico (网站图标)
     * - 公共目录下的静态资源
     */
    '/((?!api|_next/static|_next/image|favicon.ico|.*\\..*|_next).*)',
  ],
};
