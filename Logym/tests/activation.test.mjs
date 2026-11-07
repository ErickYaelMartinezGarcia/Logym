import test from 'node:test'
import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import { SourceTextModule, SyntheticModule, createContext } from 'node:vm'
import { parse, compileScript } from '@vue/compiler-sfc'
import * as Vue from 'vue'

const request = 'LOGYM2-HW-' + 'a'.repeat(64)
async function appFixture(options = {}) {
  let valid = options.valid ?? false
  const calls = [], listeners = new Map(), copies = []
  const doc = { activeElement: null, execCommand: () => options.fallback ?? false }
  globalThis.document = doc
  const element = type => ({ type, tagName: type.toUpperCase(), children: [], props: {}, style: {}, text: '', events: {},
    addEventListener(name, fn) { this.events[name] = fn }, removeEventListener(name) { delete this.events[name] },
    focus() { doc.activeElement = this }, select() {}, setAttribute() {}, removeAttribute() {} })
  const renderer = Vue.createRenderer({
    createElement: element, createText: text => ({ ...element('#text'), text }), createComment: text => ({ ...element('#comment'), text }),
    setText: (n, text) => { n.text = text }, setElementText: (n, text) => { n.text = text; n.children = [] },
    parentNode: n => n.parent, nextSibling: n => { const s=n.parent?.children; return s?.[s.indexOf(n)+1] ?? null },
    insert(n, parent, anchor = null) { if(n.parent) this.remove(n); n.parent=parent; const i=parent.children.indexOf(anchor);parent.children.splice(i<0?parent.children.length:i,0,n) },
    remove(n) { const c=n.parent?.children;if(c)c.splice(c.indexOf(n),1);n.parent=null },
    patchProp(n, key, before, after) { n.props[key]=after; if(key==='value')n.value=after },
  })
  const context=createContext({console,document:doc,navigator:{clipboard:{writeText:async code=>{if(options.clipboardError)throw Error('denied');copies.push(code)}}},setInterval:()=>1,clearInterval:()=>{},setTimeout,clearTimeout})
  const invoke=async (command,args)=>{calls.push([command,args]);if(command==='estado_licencia'){if(options.statusError)throw Error('IPC no disponible');return valid}if(command==='solicitud_licencia'){if(options.hardwareError)throw Error('UUID SMBIOS genérico');return request}if(command==='activar_licencia'){if(options.activationError)throw options.activationError;valid=true;return}if(command==='estado_lector')return {estado:'disponible',revision:1};return 'ok'}
  const mocks={vue:Vue,'@tauri-apps/api/tauri':{invoke},'@tauri-apps/api/event':{listen:async(name,fn)=>{listeners.set(name,fn);return()=>listeners.delete(name)}},'vue-router':{useRoute:()=>({path:'/',name:'Check-in'})}}
  const cache=new Map()
  async function load(file) {
    if(cache.has(file))return cache.get(file)
    const source=await readFile(new URL('../src/'+file,import.meta.url),'utf8')
    const {descriptor}=parse(source)
    const compiled=compileScript(descriptor,{id:file,inlineTemplate:true})
    const module=new SourceTextModule(compiled.content,{context,identifier:file});cache.set(file,module)
    await module.link(async spec=>{
      let values=mocks[spec]
      if(spec.endsWith('ToastNotifications.vue'))values={default:{render:()=>null}}
      if(spec.endsWith('/toast'))values={mostrarToast:()=>{}}
      if(spec.endsWith('.svg'))values={default:'synthetic.svg'}
      if(values)return new SyntheticModule(Object.keys(values),function(){for(const[k,v]of Object.entries(values))this.setExport(k,v)},{context})
      if(spec.endsWith('.vue'))return load(new URL(spec,'file:///'+file).pathname.slice(1))
      throw Error('Import no simulado: '+spec)
    });await module.evaluate();return module
  }
  const Root=(await load('App.vue')).namespace.default
  const Shell=(await load('components/AppShell.vue')).namespace.default
  const application=renderer.createApp(Root)
  application.component('router-view',{setup(){Vue.onMounted(()=>invoke('inicializar_db'));return()=>Vue.h(Shell,null,{default:()=>Vue.h('p','Sistema de prueba')})}})
  application.component('router-link',{props:['to'],setup(_, {slots}){return()=>Vue.h('a',slots.default?.())}})
  const root=element('root');application.mount(root)
  const flush=async()=>{for(let i=0;i<8;i++){await Promise.resolve();await Vue.nextTick()}}
  const nodes=(n=root)=>[n,...n.children.flatMap(c=>nodes(c))]
  const text=(n=root)=>n.type==='#comment'?'':n.text+n.children.map(c=>text(c)).join(' ')
  const button=label=>nodes().find(n=>n.type==='button'&&(text(n).includes(label)||n.props['aria-label']===label))
  const click=async label=>{const b=button(label);assert(b,'Botón visible en render: '+label);await b.props.onClick?.();await flush()}
  const submit=async(value='signed synthetic license')=>{const input=nodes().find(n=>n.props.id==='licencia');assert(input);input.value=value;input.events.input({target:input});await flush();const form=nodes().find(n=>n.type==='form');await form.props.onSubmit({preventDefault(){}});await flush()}
  await flush()
  const openLicense=async()=>{assert(!button('Licencia de esta computadora'));await click('Abrir menú');await click('Información del sistema');const dot=nodes().find(n=>n.type==='span'&&n.props['aria-label']==='Consultar licencia');assert(dot,'Punto verde dentro de información');await dot.props.onClick();await flush();assert(!text().includes('Acerca de Logym'))}
  return {calls,listeners,copies,nodes,text,button,click,submit,flush,openLicense,unmount:()=>application.unmount()}
}

test('Arranque sin licencia muestra activación y hardware; no monta rutas ni abre DB',async()=>{
 const f=await appFixture();try{assert(f.text().includes('Activación de Logym'));assert(f.nodes().some(n=>n.props.id==='solicitud'&&n.props.value===request));assert(f.button('Activar licencia'));assert(!f.calls.some(([c])=>c==='inicializar_db'||c==='estado_lector'));assert.equal(f.listeners.size,0);await f.click('Copiar código');assert.deepEqual(f.copies,[request])}finally{f.unmount()}
})
test('Activación correcta entra sin reiniciar; licencia consultable desde Información del sistema',async()=>{
 const f=await appFixture();try{await f.submit();assert(f.text().includes('Sistema de prueba'));assert(!f.text().includes('Activación de Logym'));assert.equal(f.calls.filter(([c])=>c==='inicializar_db').length,1);await f.openLicense();assert(f.text().includes('Licencia válida'));assert(f.button('Volver al sistema'));await f.click('Volver al sistema');assert.equal(f.calls.filter(([c])=>c==='inicializar_db').length,1)}finally{f.unmount()}
})
for(const error of ['Firma de licencia inválida','Licencia de otra computadora'])test('Activación rechazada visible: '+error,async()=>{
 const f=await appFixture({activationError:error});try{await f.submit();assert(f.text().includes(error));assert(!f.calls.some(([c])=>c==='inicializar_db'));assert(f.button('Activar licencia'));assert.equal(f.button('Activar licencia').props.disabled,false)}finally{f.unmount()}
})
test('Licencia válida inicia directamente y permite consultar/copiar hardware',async()=>{
 const f=await appFixture({valid:true});try{assert(f.text().includes('Sistema de prueba'));assert(!f.nodes().some(n=>n.props.id==='licencia'));await f.openLicense();assert(f.text().includes('Licencia válida'));await f.click('Copiar código');assert.deepEqual(f.copies,[request])}finally{f.unmount()}
})
test('Fallo de verificación o SMBIOS mantiene bloqueo y muestra la causa',async()=>{
 for(const options of [{statusError:true},{hardwareError:true}]){const f=await appFixture(options);try{assert(!f.calls.some(([c])=>c==='inicializar_db'));assert(f.text().includes(options.statusError?'IPC no disponible':'UUID SMBIOS genérico'));if(options.hardwareError)assert.equal(f.button('Activar licencia').props.disabled,true)}finally{f.unmount()}}
})
test('Copiar funciona con fallback y muestra instrucción si clipboard falla',async()=>{
 for(const fallback of [true,false]){const f=await appFixture({clipboardError:true,fallback});try{await f.click('Copiar código');assert(f.text().includes(fallback?'Código copiado':'cópialo manualmente'))}finally{f.unmount()}}
})
