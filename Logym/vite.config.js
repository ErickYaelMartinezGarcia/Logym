// import { defineConfig } from 'vite'
// import vue from '@vitejs/plugin-vue'

// // https://vite.dev/config/
// export default defineConfig({
//   plugins: [vue()],
//   resolve: {
//     alias: {
//       'vue': 'vue/dist/vue.esm-bundler.js'
//     }
//   }
// });

//
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import federation from '@originjs/vite-plugin-federation'

export default defineConfig({
  plugins: [
    vue(),
    federation({
      name: 'logyMf',
      filename: 'remoteEntry.js',
      exposes: {
        './Login': './src/Login.vue'
      },
      shared: ['vue']
    })
  ],
  build: {
    target: 'esnext'
  },
  resolve: {
    alias: {
      'vue': 'vue/dist/vue.esm-bundler.js'
    }
  }
})
