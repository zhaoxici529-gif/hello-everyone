import { convertFileSrc } from '@tauri-apps/api/core'

/**
 * 把磁盘路径转成 WebView 能直接加载的地址（asset 协议）。
 * 浏览器预览下返回空串，调用方显示占位图。
 */
export function fileUrl(path: string | null | undefined): string {
  if (!path) return ''
  if (typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) return ''

  try {
    return convertFileSrc(path)
  } catch {
    return ''
  }
}
