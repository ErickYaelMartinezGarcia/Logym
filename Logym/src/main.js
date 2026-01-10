import { createApp } from 'vue'
import AppComponent from './App.js';


const app = createApp({data() {return {}},methods: {},mounted() {},watch: {}});


app.component('app-component', AppComponent);

app.mount('#app');
