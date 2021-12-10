<template>
  <div>Drosse UI</div>
</template>

<script>
import { invoke } from '@tauri-apps/api/tauri'
import { listen } from '@tauri-apps/api/event'

export default {
  name: 'App',
  setup() {
    invoke('init_discover')

    listen('awesome', e => {
      console.log(e.payload.message)
    })
  }
}
</script>

<style>
#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  color: #2c3e50;
  margin-top: 60px;
}
</style>
