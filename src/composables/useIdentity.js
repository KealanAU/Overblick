import { ref, watch } from 'vue'

const USERNAME_KEY = 'overblick:username'
const TOKEN_KEY    = 'overblick:github_token'

const username    = ref(localStorage.getItem(USERNAME_KEY) || '')
const githubToken = ref(localStorage.getItem(TOKEN_KEY)    || '')

watch(username, (v) => {
  if (v) localStorage.setItem(USERNAME_KEY, v)
  else   localStorage.removeItem(USERNAME_KEY)
})

watch(githubToken, (v) => {
  if (v) localStorage.setItem(TOKEN_KEY, v)
  else   localStorage.removeItem(TOKEN_KEY)
})

export function useIdentity() {
  return { username, githubToken }
}
