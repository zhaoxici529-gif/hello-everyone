import type { AiErrorPayload } from '../types'

/** 浏览器预览（没有 Tauri 运行时）时的统一错误 */
const NO_RUNTIME: AiErrorPayload = {
  code: 'no_runtime',
  message: '当前是浏览器预览模式，这个功能需要桌面应用',
  hint: '请双击「自媒体AI工作台.exe」运行，或在项目目录执行 pnpm tauri dev。',
}

export function isAiError(value: unknown): value is AiErrorPayload {
  return Boolean(value && typeof value === 'object' && 'code' in value && 'message' in value)
}

/** 把各种异常归一成带 code 的友好错误 */
export function normalizeAiError(error: unknown): AiErrorPayload {
  if (isAiError(error)) return error
  if (typeof error === 'string') {
    return { code: 'internal_error', message: error, hint: '' }
  }
  return {
    code: 'internal_error',
    message: '调用失败，请稍后重试',
    hint: '',
  }
}

/**
 * 调用 Tauri command 并把后端返回的结构化错误原样抛出。
 * AI 相关的命令都返回 Result<T, AiError>，所以前端能拿到 code / message / hint。
 */
export async function invokeCommand<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) {
    throw NO_RUNTIME
  }

  const { invoke } = await import('@tauri-apps/api/core')
  try {
    return await invoke<T>(command, args)
  } catch (error) {
    throw normalizeAiError(error)
  }
}
