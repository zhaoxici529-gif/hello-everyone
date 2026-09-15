/**
 * 用浏览器能力抓视频首帧做缩略图。
 * 这样不用打包 ffmpeg，代价是：编码不被 WebView 支持、或画布被跨域污染时抓不到，
 * 此时素材会退化成「有播放按钮的占位图」，不影响使用。
 */
export async function captureVideoFrame(filePath: string): Promise<string | null> {
  if (typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) return null

  const { convertFileSrc } = await import('@tauri-apps/api/core')
  const video = document.createElement('video')
  video.src = convertFileSrc(filePath)
  video.crossOrigin = 'anonymous'
  video.muted = true
  video.preload = 'metadata'

  try {
    await new Promise<void>((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error('读取视频超时')), 8000)
      video.onloadeddata = () => {
        clearTimeout(timer)
        resolve()
      }
      video.onerror = () => {
        clearTimeout(timer)
        reject(new Error('浏览器无法解码这个视频'))
      }
    })

    await new Promise<void>((resolve) => {
      video.onseeked = () => resolve()
      video.currentTime = Math.min(0.5, (video.duration || 1) / 2)
    })

    const width = video.videoWidth || 480
    const height = video.videoHeight || 270
    const scale = Math.min(1, 480 / Math.max(width, height))

    const canvas = document.createElement('canvas')
    canvas.width = Math.max(1, Math.round(width * scale))
    canvas.height = Math.max(1, Math.round(height * scale))

    const context = canvas.getContext('2d')
    if (!context) return null

    context.drawImage(video, 0, 0, canvas.width, canvas.height)
    return canvas.toDataURL('image/jpeg', 0.8)
  } finally {
    video.removeAttribute('src')
    video.load()
  }
}
