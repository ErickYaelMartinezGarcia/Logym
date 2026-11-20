// Mock de la API de escritorio exclusivo de demo/vercel. No importa Tauri ni Rust.
// Todo el estado vive en memoria y se reinicia con cada recarga de la página.
import { emit } from './eventMock.js'

const fecha = (d = new Date()) => [d.getFullYear(), String(d.getMonth()+1).padStart(2,'0'), String(d.getDate()).padStart(2,'0')].join('-')
const mover = (date, dias) => { const d = new Date(date+'T12:00:00'); d.setDate(d.getDate()+dias); return fecha(d) }
const hoy = () => fecha()
const planes = [
  {id:1,nombre:'Mensual',duracion_meses:1,precio_centavos:35000,activo:true,solo_alta:false},
  {id:2,nombre:'Estudiante',duracion_meses:1,precio_centavos:28000,activo:true,solo_alta:false},
  {id:3,nombre:'Pareja',duracion_meses:1,precio_centavos:30000,activo:true,solo_alta:false},
  {id:4,nombre:'Semestral',duracion_meses:6,precio_centavos:190000,activo:true,solo_alta:false}
]
let inscripcion = 10000
const meses = (inicio,n) => {const d=new Date(inicio+'T12:00:00'); const dia=d.getDate();d.setDate(1);d.setMonth(d.getMonth()+n);const ultimo=new Date(d.getFullYear(),d.getMonth()+1,0).getDate();d.setDate(Math.min(dia,ultimo));return fecha(d)}
let clientes = [
  ['Alex Rivera',1,-16],['Mariana López',2,-8],['Carlos Mendoza',1,-34],
  ['Sofía Hernández',3,-10],['Diego Torres',1,-40],['Valeria Cruz',1,-4],
  ['Luis Ramírez',4,-75],['Fernanda Ruiz',2,-19]
].map(([nombre,tipo,offset],i)=>{
 const inicio=mover(hoy(),offset)
 return {id:i+1,nombre,telefono:'',telefono_emergencia:'',fecha_registro:inicio,fecha_inicio_membresia:inicio,fecha_fin_membresia:meses(inicio,planes.find(p=>p.id===tipo).duracion_meses),tipo_membresia_id:tipo,activo:true,foto_base64:'',template_huella:''}
})
let pagos = clientes.map((c,i)=>({id:i+1,fecha:c.fecha_registro,total_centavos:planes.find(p=>p.id===c.tipo_membresia_id).precio_centavos,membresias_centavos:planes.find(p=>p.id===c.tipo_membresia_id).precio_centavos,inscripciones_centavos:0,tipo:'alta'}))
let asistencias=[{cliente_id:1,fecha:hoy(),hora:'08'},{cliente_id:2,fecha:hoy(),hora:'09'},{cliente_id:4,fecha:hoy(),hora:'18'}]
let contadorHuella=0
const emitirCambio=()=>emit('logym://datos-actualizados',{})
const planDe=id=>planes.find(p=>p.id===Number(id))||planes[0]
const diasRestantes=c=>Math.round((new Date(c.fecha_fin_membresia+'T12:00:00')-new Date(hoy()+'T12:00:00'))/86400000)
const mostrar=c=>{
 const dias=diasRestantes(c)
 const estado=!c.fecha_fin_membresia?'sin_fecha':!c.activo?'vencida':c.fecha_inicio_membresia>hoy()?'pendiente':dias<0?'vencida':dias<=7?'por_vencer':'activa'
 return {...c,tipo_membresia:planDe(c.tipo_membresia_id).nombre,estado_membresia:estado,dias_restantes:dias}
}
function normalizar(c){return mostrar(c)}
function guardar(datos){
 const plan=planDe(datos.tipoMembresiaId)
 if(!datos.nombre?.trim()) throw Error('Ingresa el nombre')
 const c={id:Math.max(0,...clientes.map(c=>c.id))+1,nombre:datos.nombre.trim(),telefono:datos.telefono||'',telefono_emergencia:datos.telefonoEmergencia||'',fecha_registro:datos.fechaRegistro||hoy(),fecha_inicio_membresia:datos.fechaInicioMembresia||hoy(),fecha_fin_membresia:datos.fechaFinMembresia||meses(hoy(),plan.duracion_meses),tipo_membresia_id:plan.id,activo:true,foto_base64:datos.fotoBase64||'',template_huella:'demo'}
 clientes.unshift(c)
 const alta=datos.incluirInscripcion?inscripcion:0
 pagos.unshift({id:Date.now(),fecha:hoy(),tipo:'alta',total_centavos:plan.precio_centavos+alta,membresias_centavos:plan.precio_centavos,inscripciones_centavos:alta})
 emitirCambio()
 return c.id
}
function renovar(id,planId){
 const c=clientes.find(c=>c.id===Number(id));if(!c)throw Error('Cliente no encontrado')
 const p=planDe(planId),inicio=c.fecha_fin_membresia>=hoy()?mover(c.fecha_fin_membresia,1):hoy()
 c.fecha_inicio_membresia=inicio;c.fecha_fin_membresia=meses(inicio,p.duracion_meses);c.tipo_membresia_id=p.id;c.activo=true
 pagos.unshift({id:Date.now(),fecha:hoy(),tipo:'renovacion',total_centavos:p.precio_centavos,membresias_centavos:p.precio_centavos,inscripciones_centavos:0})
 emitirCambio();return mostrar(c)
}
function registrarAsistencia(id){
 const c=clientes.find(c=>c.id===Number(id));if(!c)throw Error('Cliente no encontrado')
 const info=mostrar(c)
 if(info.estado_membresia==='pendiente')return {cliente:info,resultado:'membresia_pendiente'}
 if(info.estado_membresia==='vencida'||info.estado_membresia==='sin_fecha')return {cliente:info,resultado:'membresia_vencida'}
 if(asistencias.some(a=>a.cliente_id===c.id&&a.fecha===hoy()))return {cliente:info,resultado:'duplicada'}
 asistencias.unshift({cliente_id:c.id,fecha:hoy(),hora:String(new Date().getHours()).padStart(2,'0')})
 emitirCambio();return {cliente:info,resultado:'registrada'}
}
function financiera(inicio,fin){
 const relevantes=pagos.filter(p=>p.fecha>=inicio&&p.fecha<=fin)
 const series=new Map()
 for(const p of relevantes)series.set(p.fecha,(series.get(p.fecha)||0)+p.total_centavos)
 return {fecha_inicio:inicio,fecha_fin:fin,numero_pagos:relevantes.length,total_centavos:relevantes.reduce((a,p)=>a+p.total_centavos,0),membresias_centavos:relevantes.reduce((a,p)=>a+p.membresias_centavos,0),inscripciones_centavos:relevantes.reduce((a,p)=>a+p.inscripciones_centavos,0),serie:[...series].map(([fecha,total_centavos])=>({fecha,total_centavos})).sort((a,b)=>a.fecha.localeCompare(b.fecha))}
}
export async function invoke(comando,args={}){
 switch(comando){
 case 'inicializar_db':return true
 case 'estado_licencia':return true
 case 'estado_lector':return {estado:'disponible',detalle:'Simulación sin dispositivo',revision:1,usb_presente:false,sdk_inicializado:false,captura_comprobada:false}
 case 'pausar_lector_global':return true
 case 'listar_clientes':return clientes.map(c=>({...c}))
 case 'listar_clientes_membresia':return clientes.map(mostrar)
 case 'listar_tipos_membresia':return planes.filter(p=>p.activo).map(p=>({...p}))
 case 'obtener_precio_inscripcion':return inscripcion
 case 'guardar_cliente':return guardar(args.cliente)
 case 'capturar_huella':return (++contadorHuella%2?'a':'b').repeat(800)
 case 'comparar_huella':return true // Las huellas de la demo no son biometría real
 case 'obtener_metricas':return {total_clientes:clientes.filter(c=>c.activo).length,asistencias_hoy:asistencias.filter(a=>a.fecha===hoy()).length,nuevos_semana:clientes.filter(c=>c.fecha_registro>=mover(hoy(),-6)).length,huella_activas:clientes.length,asistencias_ultimos_7_dias:asistencias.filter(a=>a.fecha>=mover(hoy(),-6)).length,dia_mas_concurrido:hoy(),hora_mayor_afluencia:'18:00'}
 case 'obtener_asistencias_diarias':{const mapa=new Map();for(const a of asistencias){let clave=args.periodo==='mes'?a.fecha.slice(0,7):a.fecha;mapa.set(clave,(mapa.get(clave)||0)+1)}return [...mapa].map(([fecha,total])=>({fecha,total}))}
 case 'obtener_asistencias_por_hora':{const mapa=new Map();for(const a of asistencias.filter(a=>a.fecha===hoy()))mapa.set(a.hora,(mapa.get(a.hora)||0)+1);return [...mapa].map(([fecha,total])=>({fecha,total}))}
 case 'obtener_asistencias_periodo':return asistencias.map(a=>({...a}))
 case 'actualizar_cliente':{const c=clientes.find(c=>c.id===Number(args.id));if(!c)throw Error('Cliente no encontrado');const d=args.datos;c.nombre=d.nombre;c.telefono=d.telefono;c.telefono_emergencia=d.telefonoEmergencia;c.fecha_inicio_membresia=d.fechaInicioMembresia;c.fecha_fin_membresia=d.fechaFinMembresia;emitirCambio();return true}
 case 'establecer_cliente_activo':{const c=clientes.find(c=>c.id===Number(args.id));if(!c)throw Error('Cliente no encontrado');c.activo=!!args.activo;emitirCambio();return true}
 case 'eliminar_cliente_definitivamente':{if(args.confirmacion!=='ELIMINAR')throw Error('Confirma ELIMINAR');clientes=clientes.filter(c=>c.id!==Number(args.id));emitirCambio();return true}
 case 'renovar_membresia':return renovar(args.datos.clienteId,args.datos.tipoMembresiaId)
 case 'registrar_asistencia':return registrarAsistencia(args.clienteId).resultado
 case 'demo_checkin':{const c=clientes.find(c=>c.id===Number(args.clienteId))||clientes.find(c=>c.activo);if(!c)throw Error('No hay clientes');const e=registrarAsistencia(c.id);emit('logym://checkin',e);return e}
 case 'admin_configurado':return true
 case 'crear_password_admin':case 'validar_password_admin':return true
 case 'listar_planes_admin':return planes.map(p=>({...p}))
 case 'guardar_plan_admin':{const p=planes.find(p=>p.id===Number(args.id));if(p){Object.assign(p,{nombre:args.nombre,duracion_meses:args.meses,precio_centavos:args.precioCentavos,activo:args.activo})}else{planes.push({id:Math.max(...planes.map(p=>p.id))+1,nombre:args.nombre,duracion_meses:args.meses,precio_centavos:args.precioCentavos,activo:true,solo_alta:false})}emitirCambio();return true}
 case 'guardar_inscripcion_admin':inscripcion=Number(args.precioCentavos);return true
 case 'reporte_financiero':return financiera(args.fechaInicio,args.fechaFin)
 case 'resumen_ingresos':return financiera(mover(hoy(),-30),hoy())
 default:throw Error('Función no simulada en demo: '+comando)
 }
}
export default invoke
