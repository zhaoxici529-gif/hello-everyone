import type { GlobalThemeOverrides } from 'naive-ui'

/**
 * Naive UI 深色主题覆盖：把组件库的默认深灰换成工作台这套
 * 「深绿底 + 浅绿主色」，保证第三方组件和自绘卡片观感一致。
 */
export const themeOverrides: GlobalThemeOverrides = {
  common: {
    primaryColor: '#b8d4a8',
    primaryColorHover: '#c8dfba',
    primaryColorPressed: '#8fbc8f',
    primaryColorSuppl: '#8fbc8f',
    infoColor: '#8fb8bc',
    successColor: '#8fbc8f',
    warningColor: '#d8c08a',
    errorColor: '#d79a92',

    bodyColor: '#151815',
    cardColor: '#1b201b',
    modalColor: '#1b201b',
    popoverColor: '#232823',
    tableColor: '#1b201b',
    inputColor: '#121512',

    textColorBase: '#e8ece6',
    textColor1: '#e8ece6',
    textColor2: '#aab3a7',
    textColor3: '#7d867c',

    borderColor: 'rgba(255, 255, 255, 0.12)',
    dividerColor: 'rgba(255, 255, 255, 0.07)',
    hoverColor: 'rgba(184, 212, 168, 0.08)',

    borderRadius: '12px',
    borderRadiusSmall: '8px',
    fontFamily:
      '"PingFang SC", "Microsoft YaHei", "Hiragino Sans GB", "Noto Sans SC", system-ui, sans-serif',
  },
}
