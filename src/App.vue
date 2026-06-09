<template>
  <div id="app">
    <MainApp v-if="isMainWindow" />
    <Widget v-else />
  </div>
</template>

<script>
import Widget from './components/Widget.vue'
import MainApp from './components/MainApp.vue'
import { getCurrentWindow } from '@tauri-apps/api/window'

export default {
  components: { Widget, MainApp },
  data() {
    return {
      isMainWindow: false
    }
  },
  async mounted() {
    const win = getCurrentWindow()
    this.isMainWindow = win.label === 'main'
  }
}
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  background: transparent;
  font-family: sans-serif;
}
</style>