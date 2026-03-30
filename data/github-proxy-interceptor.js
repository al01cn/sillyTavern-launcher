
// GitHub URL 拦截器 - 自动重写 GitHub 链接到镜像
import https from 'https';
import http from 'http';
const originalHttpsRequest = https.request;
const originalHttpsGet = https.get;
const originalHttpRequest = http.request;
const originalHttpGet = http.get;

const PROXY_URL = 'https://gh.sixyin.com';

function rewriteGitHubUrl(url) {
    if (!url || typeof url !== 'string') return url;

    // 不重写 API 请求
    if (url.includes('api.github.com')) return url;

    // 只重写 GitHub 相关的 URL
    if (url.includes('github.com') || url.includes('raw.githubusercontent.com')) {
        return PROXY_URL + '/' + url;
    }

    return url;
}

// 拦截 https.request
https.request = function(url, options, callback) {
    let req;
    if (typeof url === 'string') {
        const rewrittenUrl = rewriteGitHubUrl(url);
        req = originalHttpsRequest.call(https, rewrittenUrl, options, callback);
    } else if (url && typeof url === 'object') {
        if (url.href) {
            const newUrl = Object.assign({}, url);
            newUrl.href = rewriteGitHubUrl(url.href);
            // 同步修改 host/hostname 等其他可能被使用的字段
            if (newUrl.href !== url.href) {
                try {
                    const parsed = new URL(newUrl.href);
                    newUrl.host = parsed.host;
                    newUrl.hostname = parsed.hostname;
                    newUrl.pathname = parsed.pathname;
                    newUrl.protocol = parsed.protocol;
                    newUrl.port = parsed.port;
                } catch (e) {}
            }
            req = originalHttpsRequest.call(https, newUrl, options, callback);
        } else {
            req = originalHttpsRequest.call(https, url, options, callback);
        }
    } else {
        req = originalHttpsRequest.call(https, url, options, callback);
    }

    // 拦截 request 的 write 方法，确保 URL 也被重写
    const originalWrite = req.write;
    req.write = function(chunk, encoding, callback) {
        if (chunk && typeof chunk === 'string') {
            try {
                const data = JSON.parse(chunk);
                if (data.url && typeof data.url === 'string') {
                    const rewrittenUrl = rewriteGitHubUrl(data.url);
                    if (rewrittenUrl !== data.url) {
                        data.url = rewrittenUrl;
                        chunk = JSON.stringify(data);
                    }
                }
            } catch (e) {
                // 忽略解析错误
            }
        }
        return originalWrite.call(req, chunk, encoding, callback);
    };

    return req;
};

// 拦截 https.get
https.get = function(url, options, callback) {
    if (typeof url === 'string') {
        const rewrittenUrl = rewriteGitHubUrl(url);
        return originalHttpsGet.call(https, rewrittenUrl, options, callback);
    } else if (url && typeof url === 'object') {
        if (url.href) {
            const newUrl = Object.assign({}, url);
            newUrl.href = rewriteGitHubUrl(url.href);
            if (newUrl.href !== url.href) {
                try {
                    const parsed = new URL(newUrl.href);
                    newUrl.host = parsed.host;
                    newUrl.hostname = parsed.hostname;
                    newUrl.pathname = parsed.pathname;
                    newUrl.protocol = parsed.protocol;
                    newUrl.port = parsed.port;
                } catch (e) {}
            }
            return originalHttpsGet.call(https, newUrl, options, callback);
        }
        return originalHttpsGet.call(https, url, options, callback);
    }
    return originalHttpsGet.call(https, url, options, callback);
};

// 拦截 http.request（部分 GitHub 资源可能使用 HTTP）
http.request = function(url, options, callback) {
    if (typeof url === 'string') {
        const rewrittenUrl = rewriteGitHubUrl(url);
        return originalHttpRequest.call(http, rewrittenUrl, options, callback);
    } else if (url && typeof url === 'object') {
        if (url.href) {
            const newUrl = Object.assign({}, url);
            newUrl.href = rewriteGitHubUrl(url.href);
            if (newUrl.href !== url.href) {
                try {
                    const parsed = new URL(newUrl.href);
                    newUrl.host = parsed.host;
                    newUrl.hostname = parsed.hostname;
                    newUrl.pathname = parsed.pathname;
                    newUrl.protocol = parsed.protocol;
                    newUrl.port = parsed.port;
                } catch (e) {}
            }
            return originalHttpRequest.call(http, newUrl, options, callback);
        }
        return originalHttpRequest.call(http, url, options, callback);
    }
    return originalHttpRequest.call(http, url, options, callback);
};

// 拦截 http.get
http.get = function(url, options, callback) {
    if (typeof url === 'string') {
        const rewrittenUrl = rewriteGitHubUrl(url);
        return originalHttpGet.call(http, rewrittenUrl, options, callback);
    } else if (url && typeof url === 'object') {
        if (url.href) {
            const newUrl = Object.assign({}, url);
            newUrl.href = rewriteGitHubUrl(url.href);
            if (newUrl.href !== url.href) {
                try {
                    const parsed = new URL(newUrl.href);
                    newUrl.host = parsed.host;
                    newUrl.hostname = parsed.hostname;
                    newUrl.pathname = parsed.pathname;
                    newUrl.protocol = parsed.protocol;
                    newUrl.port = parsed.port;
                } catch (e) {}
            }
            return originalHttpGet.call(http, newUrl, options, callback);
        }
        return originalHttpGet.call(http, url, options, callback);
    }
    return originalHttpGet.call(http, url, options, callback);
};

console.log('[GitHub Proxy] URL interceptor loaded, proxy:', PROXY_URL);
