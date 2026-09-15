import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// Tauri 期望前端固定在 1420 端口，且不要清屏，否则会盖掉 Rust 侧日志。
export default defineConfig({
  plugins: [vue()],
  clearScreen: false,
  // 相对路径：这样 dist/index.html 直接双击用浏览器打开也不会白屏，
  // Tauri 打包后的 tauri:// 协议下同样正常。
  base: './',
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // src-tauri 由 cargo 自己监听，避免重复触发前端热更新。
      ignored: ['**/src-tauri/**'],
    },
  },
  build: {
    target: 'chrome110',
    sourcemap: false,
  },
})
