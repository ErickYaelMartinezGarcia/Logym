const subs = new Map()
export async function listen(event, callback) {
  if (!subs.has(event)) subs.set(event,new Set())
  subs.get(event).add(callback)
  return () => subs.get(event)?.delete(callback)
}
export function emit(event,payload){for(const cb of [...(subs.get(event)||[])])cb({event,payload})}
