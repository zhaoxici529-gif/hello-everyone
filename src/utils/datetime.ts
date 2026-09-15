function pad(value: number): string {
  return String(value).padStart(2, '0')
}

/** 本地日期 YYYY-MM-DD */
export function localDate(date: Date = new Date()): string {
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`
}

/** 本地时间 YYYY-MM-DD HH:mm:ss，与 SQLite 的 datetime('now','localtime') 格式一致 */
export function localDateTime(date: Date = new Date()): string {
  return `${localDate(date)} ${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`
}
