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
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [
    tailwindcss(),
    vue(),
    federation({
      name: 'logyMf',
      filename: 'remoteEntry.js',
      exposes: {
        './Login': './src/App.js'
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
