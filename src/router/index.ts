import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router'
import AppLayout from '../layouts/AppLayout.vue'

export interface NavItem {
  /** 路由路径 */
  path: string
  /** 菜单显示名 */
  title: string
  /** 图标名，对应 AppIcon 的 name */
  icon: string
  /** 副标题，用于占位页说明 */
  caption: string
}

/** 顶部主导航：从左到右 7 项。 */
export const navItems: NavItem[] = [
  { path: '/flow', title: '内容流程', icon: 'flow', caption: '一条内容从灵感到发布的完整流转' },
  { path: '/', title: '首页', icon: 'home', caption: '今天的全局视角' },
  { path: '/trends', title: '今日热点', icon: 'flame', caption: '每天自动汇总的行业热点' },
  { path: '/topics', title: '选题', icon: 'bulb', caption: '把热点、课程、评论变成可写的选题' },
  { path: '/create', title: '创作', icon: 'pen', caption: '白板 + 初稿 + 改稿的创作现场' },
  { path: '/publish', title: '发布与经营', icon: 'chart', caption: '发布台账与数据复盘' },
  { path: '/plan', title: '计划', icon: 'calendar', caption: '今天、本周、本月的排期与目标' },
]

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    component: AppLayout,
    children: [
      { path: '', name: 'home', component: () => import('../views/HomeView.vue') },
      { path: 'flow', name: 'flow', component: () => import('../views/FlowView.vue') },
      { path: 'trends', name: 'trends', component: () => import('../views/TrendsView.vue') },
      { path: 'topics', name: 'topics', component: () => import('../views/TopicsView.vue') },
      { path: 'create', name: 'create', component: () => import('../views/CreateView.vue') },
      { path: 'publish', name: 'publish', component: () => import('../views/PublishView.vue') },
      { path: 'plan', name: 'plan', component: () => import('../views/PlanView.vue') },
      { path: 'personas', name: 'personas', component: () => import('../views/PersonasView.vue') },
      { path: 'materials', name: 'materials', component: () => import('../views/MaterialsView.vue') },
      {
        path: 'manuscripts',
        name: 'manuscripts',
        component: () => import('../views/ManuscriptsView.vue'),
      },
      { path: 'memory', name: 'memory', component: () => import('../views/MemoryView.vue') },
      { path: 'settings', name: 'settings', component: () => import('../views/SettingsView.vue') },
    ],
  },
  { path: '/:pathMatch(.*)*', redirect: '/' },
]

export default createRouter({
  // 桌面端走 hash 模式，打包后 file:// 协议下也能正常跳转。
  history: createWebHashHistory(),
  routes,
})
