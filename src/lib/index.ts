export const openUrl = (url: string) => {
    (window as any).system.openBrowser(url)
}

export async function getVersion() {
    const version = await (window as any).system.getVersion()
    return version
}