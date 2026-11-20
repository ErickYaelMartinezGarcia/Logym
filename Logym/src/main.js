import { createApp } from 'vue'
import DemoRoot from './DemoRoot.vue'
import router from './router'
import './style.css'

// Vistas y Tailwind originales; solo el origen de datos es una simulación local.
createApp(DemoRoot).use(router).mount('#app')
