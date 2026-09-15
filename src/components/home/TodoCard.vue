<script setup lang="ts">
import { computed } from 'vue'
import AppIcon from '../AppIcon.vue'
import WorkspaceCard from '../WorkspaceCard.vue'
import { useTasksStore } from '../../stores/tasks'

const tasks = useTasksStore()

const progressStyle = computed(() => ({ width: `${tasks.progress}%` }))

async function submit(): Promise<void> {
  await tasks.add()
}
</script>

<template>
  <WorkspaceCard id="home-todo" title="今日待办" subtitle="只放今天真的要推进的事" icon="check" class="todo">
    <template #actions>
      <span class="todo-count num">{{ tasks.doneCount }} / {{ tasks.total }}</span>
    </template>

    <div class="progress">
      <div class="progress-track">
        <div class="progress-fill" :style="progressStyle" />
      </div>
      <span class="progress-text num">{{ tasks.progress }}%</span>
    </div>

    <ul class="todo-list">
      <li
        v-for="task in tasks.tasks"
        :key="task.id"
        :class="{ 'is-done': task.status === 'done' }"
      >
        <button
          class="checkbox"
          :class="{ 'is-checked': task.status === 'done' }"
          type="button"
          :aria-pressed="task.status === 'done'"
          @click="tasks.toggle(task.id)"
        >
          <AppIcon v-if="task.status === 'done'" name="check" :size="12" />
        </button>
        <span class="todo-title">{{ task.title }}</span>
        <button class="todo-remove" type="button" title="删除" @click="tasks.remove(task.id)">
          <AppIcon name="close" :size="13" />
        </button>
      </li>
      <li v-if="tasks.total === 0" class="todo-empty">今天还没有待办，先加一条最重要的。</li>
    </ul>

    <form class="todo-form" @submit.prevent="submit">
      <input
        class="field"
        :value="tasks.draft"
        placeholder="添加一条待办，回车确认"
        @input="tasks.setDraft(($event.target as HTMLInputElement).value)"
      />
      <button class="btn btn-primary btn-sm" type="submit">添加</button>
    </form>

    <div class="todo-links">
      <RouterLink to="/plan" class="btn btn-sm btn-ghost">
        <AppIcon name="list" :size="14" />
        查看完成记录（{{ tasks.doneCount }} 项）
      </RouterLink>
      <RouterLink to="/plan" class="btn btn-sm btn-ghost">
        <AppIcon name="calendar" :size="14" />
        查看全部计划
      </RouterLink>
    </div>
  </WorkspaceCard>
</template>

<style scoped>
.todo {
  height: 100%;
}

.todo-count {
  font-size: 12.5px;
  color: var(--text-3);
}

.progress {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 12px;
}

.progress-track {
  flex: 1;
  height: 6px;
  border-radius: var(--radius-pill);
  background: var(--bg-inset);
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  border-radius: var(--radius-pill);
  background: linear-gradient(90deg, var(--accent), var(--accent-strong));
  transition: width 0.28s ease;
}

.progress-text {
  font-size: 12px;
  color: var(--accent);
  min-width: 34px;
  text-align: right;
}

.todo-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
  margin: 0 0 14px;
  padding: 0;
  list-style: none;
  max-height: 208px;
  overflow-y: auto;
}

.todo-list li {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 9px 11px;
  border: 1px solid var(--border);
  border-radius: 9px;
  background: var(--bg-inset);
}

.todo-list li.is-done .todo-title {
  color: var(--text-4);
  text-decoration: line-through;
}

.checkbox {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  border: 1.5px solid var(--border-strong);
  border-radius: 5px;
  background: transparent;
  color: var(--accent-ink);
  cursor: pointer;
  flex: none;
  transition: background 0.16s ease, border-color 0.16s ease;
}

.checkbox.is-checked {
  background: var(--accent);
  border-color: var(--accent);
}

.todo-title {
  flex: 1;
  min-width: 0;
  font-size: 13px;
  color: var(--text-1);
  line-height: 1.5;
}

.todo-remove {
  border: none;
  background: transparent;
  color: var(--text-4);
  cursor: pointer;
  padding: 2px;
  border-radius: 4px;
  opacity: 0;
  transition: opacity 0.16s ease, color 0.16s ease;
}

.todo-list li:hover .todo-remove {
  opacity: 1;
}

.todo-remove:hover {
  color: var(--danger);
}

.todo-empty {
  justify-content: center;
  color: var(--text-4);
  font-size: 12.5px;
}

.todo-form {
  display: flex;
  gap: 8px;
}

.todo-links {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
  margin-top: 10px;
  padding-top: 10px;
  border-top: 1px dashed var(--divider);
}
</style>
