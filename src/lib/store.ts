import { writable } from "svelte/store"
export const devModeStore = writable(false)
export const devPathStore = writable("")
export const assetsURLStore = writable("")
export const devAssetsURLStore = writable("")
export const devUpdatesStore = writable(true)
