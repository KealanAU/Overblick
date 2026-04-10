import { ref, watch } from 'vue'

const PROVIDER_KEY     = 'overblick:ai_provider'
const CLAUDE_KEY_KEY   = 'overblick:ai_claude_key'
const CLAUDE_MODEL_KEY = 'overblick:ai_claude_model'
const OPENAI_KEY_KEY   = 'overblick:ai_openai_key'
const OPENAI_MODEL_KEY = 'overblick:ai_openai_model'
const OLLAMA_URL_KEY   = 'overblick:ai_ollama_url'
const OLLAMA_MODEL_KEY = 'overblick:ai_ollama_model'

const aiProvider  = ref(localStorage.getItem(PROVIDER_KEY)     || 'ollama')
const claudeKey   = ref(localStorage.getItem(CLAUDE_KEY_KEY)   || '')
const claudeModel = ref(localStorage.getItem(CLAUDE_MODEL_KEY) || 'claude-sonnet-4-6')
const openaiKey   = ref(localStorage.getItem(OPENAI_KEY_KEY)   || '')
const openaiModel = ref(localStorage.getItem(OPENAI_MODEL_KEY) || 'gpt-4o-mini')
const ollamaUrl   = ref(localStorage.getItem(OLLAMA_URL_KEY)   || 'http://localhost:11434')
const ollamaModel = ref(localStorage.getItem(OLLAMA_MODEL_KEY) || 'llama3.2')

function persist(key, r) {
  watch(r, v => {
    if (v) localStorage.setItem(key, v)
    else   localStorage.removeItem(key)
  })
}

persist(PROVIDER_KEY,     aiProvider)
persist(CLAUDE_KEY_KEY,   claudeKey)
persist(CLAUDE_MODEL_KEY, claudeModel)
persist(OPENAI_KEY_KEY,   openaiKey)
persist(OPENAI_MODEL_KEY, openaiModel)
persist(OLLAMA_URL_KEY,   ollamaUrl)
persist(OLLAMA_MODEL_KEY, ollamaModel)

export function useAiSettings() {
  return { aiProvider, claudeKey, claudeModel, openaiKey, openaiModel, ollamaUrl, ollamaModel }
}
