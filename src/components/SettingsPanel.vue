<script setup>
import { ref } from 'vue'
import { useTheme, PRIMARY_COLORS, NEUTRAL_COLORS, PALETTE } from '../composables/useTheme.js'
import { useIdentity } from '../composables/useIdentity.js'
import { useAiSettings } from '../composables/useAiSettings.js'

const { primaryColor, neutralColor, colorMode } = useTheme()
const { username, githubToken } = useIdentity()
const { aiProvider, claudeKey, claudeModel, openaiKey, openaiModel, ollamaUrl, ollamaModel } = useAiSettings()

const revealToken    = ref(false)
const revealClaudeKey = ref(false)
const revealOpenaiKey = ref(false)

const providerItems = [
  { label: 'Ollama (local)',  value: 'ollama' },
  { label: 'Claude (Anthropic)', value: 'claude' },
  { label: 'OpenAI',         value: 'openai' },
]

const modeOptions = [
  { label: 'Light',  value: 'light', icon: 'i-lucide-sun' },
  { label: 'Dark',   value: 'dark',  icon: 'i-lucide-moon' },
  { label: 'System', value: 'auto',  icon: 'i-lucide-monitor' },
]

function swatchStyle(color, selected) {
  const isSelected = selected === color
  const oklch = PALETTE[color]?.[5] ?? 'transparent'
  return {
    backgroundColor: oklch,
    boxShadow: isSelected
      ? `0 0 0 2px var(--ui-bg-elevated), 0 0 0 4px ${oklch}`
      : 'none',
    transform: isSelected ? 'scale(1.15)' : '',
  }
}
</script>

<template>
  <div class="p-6 max-w-xl space-y-8">
    <!-- Account -->
    <section class="space-y-4">
      <h3 class="text-xs font-semibold text-muted uppercase tracking-wider">Account</h3>
      <div class="space-y-2">
        <div class="flex items-center gap-1.5">
          <p class="text-sm font-medium">Username</p>
          <UTooltip text='Used for "Mine" filters in commits, issues & PRs.'>
            <UButton icon="i-lucide-info" variant="ghost" size="xs" color="neutral" :padded="false" />
          </UTooltip>
        </div>
        <UInput
          v-model="username"
          placeholder="GitHub / git username"
          class="max-w-xs"
        />
      </div>

      <div class="space-y-2">
        <div class="flex items-center gap-1.5">
          <p class="text-sm font-medium">GitHub Personal Access Token</p>
          <UTooltip text="Used for syncing issues & PRs. Stored in localStorage — keep this machine personal. Needs repo scope (or fine-grained read access to issues/PRs).">
            <UButton icon="i-lucide-info" variant="ghost" size="xs" color="neutral" :padded="false" />
          </UTooltip>
        </div>
        <div class="relative max-w-xs group">
          <UInput
            v-model="githubToken"
            :type="revealToken ? 'text' : 'password'"
            placeholder="ghp_..."
            class="w-full"
            @mouseenter="revealToken = true"
            @mouseleave="revealToken = false"
          />
        </div>
      </div>
    </section>

    <USeparator />

    <!-- AI -->
    <section class="space-y-4">
      <h3 class="text-xs font-semibold text-muted uppercase tracking-wider">AI Summarisation</h3>

      <div class="space-y-2">
        <p class="text-sm font-medium">Provider</p>
        <USelect v-model="aiProvider" :items="providerItems" class="max-w-xs" />
      </div>

      <!-- Ollama -->
      <template v-if="aiProvider === 'ollama'">
        <div class="space-y-2">
          <p class="text-sm font-medium">Ollama base URL</p>
          <UInput v-model="ollamaUrl" placeholder="http://localhost:11434" class="max-w-xs" />
        </div>
        <div class="space-y-2">
          <p class="text-sm font-medium">Model</p>
          <UInput v-model="ollamaModel" placeholder="llama3.2" class="max-w-xs" />
        </div>
      </template>

      <!-- Claude -->
      <template v-if="aiProvider === 'claude'">
        <div class="space-y-2">
          <div class="flex items-center gap-1.5">
            <p class="text-sm font-medium">Anthropic API key</p>
            <UTooltip text="Stored in localStorage on this machine only.">
              <UButton icon="i-lucide-info" variant="ghost" size="xs" color="neutral" :padded="false" />
            </UTooltip>
          </div>
          <div class="relative max-w-xs">
            <UInput
              v-model="claudeKey"
              :type="revealClaudeKey ? 'text' : 'password'"
              placeholder="sk-ant-..."
              class="w-full"
              @mouseenter="revealClaudeKey = true"
              @mouseleave="revealClaudeKey = false"
            />
          </div>
        </div>
        <div class="space-y-2">
          <p class="text-sm font-medium">Model</p>
          <UInput v-model="claudeModel" placeholder="claude-sonnet-4-6" class="max-w-xs" />
        </div>
      </template>

      <!-- OpenAI -->
      <template v-if="aiProvider === 'openai'">
        <div class="space-y-2">
          <div class="flex items-center gap-1.5">
            <p class="text-sm font-medium">OpenAI API key</p>
            <UTooltip text="Stored in localStorage on this machine only.">
              <UButton icon="i-lucide-info" variant="ghost" size="xs" color="neutral" :padded="false" />
            </UTooltip>
          </div>
          <div class="relative max-w-xs">
            <UInput
              v-model="openaiKey"
              :type="revealOpenaiKey ? 'text' : 'password'"
              placeholder="sk-..."
              class="w-full"
              @mouseenter="revealOpenaiKey = true"
              @mouseleave="revealOpenaiKey = false"
            />
          </div>
        </div>
        <div class="space-y-2">
          <p class="text-sm font-medium">Model</p>
          <UInput v-model="openaiModel" placeholder="gpt-4o-mini" class="max-w-xs" />
        </div>
      </template>
    </section>

    <USeparator />

    <!-- Appearance -->
    <section class="space-y-6">
      <h3 class="text-xs font-semibold text-muted uppercase tracking-wider">Appearance</h3>

      <div class="space-y-2">
        <p class="text-sm font-medium">Theme</p>
        <div class="flex gap-2">
          <UButton
            v-for="opt in modeOptions"
            :key="opt.value"
            size="sm"
            :variant="colorMode === opt.value ? 'solid' : 'outline'"
            :icon="opt.icon"
            :label="opt.label"
            @click="colorMode = opt.value"
          />
        </div>
      </div>

      <div class="space-y-2">
        <p class="text-sm font-medium">Primary colour</p>
        <div class="flex flex-wrap gap-2">
          <button
            v-for="color in PRIMARY_COLORS"
            :key="color"
            class="w-6 h-6 rounded-full transition-all cursor-pointer focus:outline-none"
            :style="swatchStyle(color, primaryColor)"
            :title="color"
            @click="primaryColor = color"
          />
        </div>
      </div>

      <div class="space-y-2">
        <p class="text-sm font-medium">Neutral colour</p>
        <div class="flex flex-wrap gap-2">
          <button
            v-for="color in NEUTRAL_COLORS"
            :key="color"
            class="w-6 h-6 rounded-full transition-all cursor-pointer focus:outline-none"
            :style="swatchStyle(color, neutralColor)"
            :title="color"
            @click="neutralColor = color"
          />
        </div>
      </div>
    </section>
  </div>
</template>
