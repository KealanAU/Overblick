<script setup>
import { ref, computed } from 'vue'

const props = defineProps({
  commits: { type: Array, required: true }
})

const today = new Date()
const currentYear = ref(today.getFullYear())
const currentMonth = ref(today.getMonth())
const selectedDay = ref(null)

const commitMap = computed(() => {
  const map = {}
  for (const c of props.commits) {
    const d = new Date(c.committed_at * 1000)
    const key = `${d.getFullYear()}-${String(d.getMonth()+1).padStart(2,'0')}-${String(d.getDate()).padStart(2,'0')}`
    if (!map[key]) map[key] = []
    map[key].push(c)
  }
  return map
})

const calendarDays = computed(() => {
  const year = currentYear.value
  const month = currentMonth.value
  const firstDay = new Date(year, month, 1)
  const daysInMonth = new Date(year, month + 1, 0).getDate()
  const offset = (firstDay.getDay() + 6) % 7
  const days = []
  for (let i = 0; i < offset; i++) days.push({ date: null, key: null })
  for (let d = 1; d <= daysInMonth; d++) {
    const date = new Date(year, month, d)
    const key = `${year}-${String(month+1).padStart(2,'0')}-${String(d).padStart(2,'0')}`
    days.push({ date, key })
  }
  while (days.length % 7 !== 0) days.push({ date: null, key: null })
  return days
})

const monthLabel = computed(() =>
  new Date(currentYear.value, currentMonth.value, 1)
    .toLocaleDateString('en', { month: 'long', year: 'numeric' })
)

const selectedDayCommits = computed(() => {
  if (!selectedDay.value) return []
  const key = `${selectedDay.value.getFullYear()}-${String(selectedDay.value.getMonth()+1).padStart(2,'0')}-${String(selectedDay.value.getDate()).padStart(2,'0')}`
  return commitMap.value[key] || []
})

function prevMonth() {
  if (currentMonth.value === 0) { currentMonth.value = 11; currentYear.value-- }
  else currentMonth.value--
  selectedDay.value = null
}
function nextMonth() {
  if (currentMonth.value === 11) { currentMonth.value = 0; currentYear.value++ }
  else currentMonth.value++
  selectedDay.value = null
}
function selectDay(day) {
  if (!day.date) return
  if (selectedDay.value?.toDateString() === day.date.toDateString()) {
    selectedDay.value = null
  } else {
    selectedDay.value = day.date
  }
}
function isToday(day) {
  return day.date?.toDateString() === today.toDateString()
}
function isSelected(day) {
  return day.date && selectedDay.value?.toDateString() === day.date.toDateString()
}
</script>

<template>
  <div class="flex flex-col h-full overflow-hidden">

    <!-- Month navigation -->
    <div class="flex items-center justify-between px-3 py-2 shrink-0">
      <UButton size="xs" variant="ghost" icon="i-lucide-chevron-left" @click="prevMonth" />
      <span class="text-sm font-semibold">{{ monthLabel }}</span>
      <UButton size="xs" variant="ghost" icon="i-lucide-chevron-right" @click="nextMonth" />
    </div>

    <!-- Day-of-week headers: Mon Tue Wed Thu Fri Sat Sun -->
    <div class="grid grid-cols-7 px-3 mb-1 shrink-0">
      <div v-for="d in ['Mon','Tue','Wed','Thu','Fri','Sat','Sun']" class="text-center text-xs text-muted py-1">{{ d }}</div>
    </div>

    <!-- Calendar grid -->
    <div class="grid grid-cols-7 gap-1 px-3 shrink-0">
      <div
        v-for="(day, i) in calendarDays"
        :key="i"
        class="aspect-square flex flex-col items-center justify-center rounded-md text-xs relative cursor-pointer select-none transition-colors"
        :class="[
          !day.date ? 'opacity-0 pointer-events-none' : 'hover:bg-elevated',
          isSelected(day) ? 'bg-primary text-white' : '',
          isToday(day) && !isSelected(day) ? 'ring-1 ring-primary' : '',
          commitMap[day.key]?.length && !isSelected(day) ? 'font-medium text-default' : 'text-muted'
        ]"
        @click="selectDay(day)"
      >
        <span>{{ day.date?.getDate() }}</span>
        <!-- Commit dot indicator -->
        <span
          v-if="day.key && commitMap[day.key]?.length"
          class="absolute bottom-1 size-1 rounded-full"
          :class="isSelected(day) ? 'bg-white' : 'bg-primary'"
        />
      </div>
    </div>

    <!-- Selected day commits panel -->
    <div v-if="selectedDay" class="mt-3 flex-1 overflow-y-auto border-t border-default">
      <div class="px-3 py-2 text-xs font-semibold text-muted bg-elevated/50 sticky top-0">
        {{ selectedDay.toLocaleDateString('en', { weekday: 'long', month: 'long', day: 'numeric' }) }}
        · {{ selectedDayCommits.length }} commit{{ selectedDayCommits.length !== 1 ? 's' : '' }}
      </div>
      <div
        v-for="commit in selectedDayCommits"
        :key="commit.id"
        class="flex items-center gap-3 px-3 py-2 border-b border-default last:border-0 hover:bg-elevated transition-colors"
      >
        <UBadge :label="commit.short_hash" variant="outline" size="sm" class="font-mono shrink-0" />
        <div class="flex-1 min-w-0">
          <p class="text-sm truncate">{{ commit.message }}</p>
          <span class="text-xs text-muted">{{ commit.author }}</span>
        </div>
      </div>
    </div>

    <!-- No day selected hint -->
    <div v-else-if="!selectedDay && commits.length" class="flex-1 flex items-center justify-center">
      <p class="text-xs text-muted">Click a day to see its commits</p>
    </div>

  </div>
</template>
