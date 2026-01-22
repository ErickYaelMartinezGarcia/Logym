import { createApp } from 'vue'
import AppComponent from './App.js';
import LoginCard  from './components/LoginCardContent/LoginCard.vue'

const app = createApp({data() {return {}},methods: {},mounted() {},watch: {}});


app.component('app-component', AppComponent);
app.component('login-card', LoginCard);

app.mount('#app');
