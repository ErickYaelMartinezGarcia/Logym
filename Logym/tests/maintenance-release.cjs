const fs=require('node:fs'),path=require('node:path'),os=require('node:os'),cp=require('node:child_process'),assert=require('node:assert/strict');
const {DatabaseSync}=require('node:sqlite');
fs.mkdirSync('.artifacts/production',{recursive:true});
const dir=fs.mkdtempSync(path.join(os.tmpdir(),'logym-production-cli-'));
const exe=path.resolve('src-tauri/target/release/logym.exe');
const original=path.join(dir,'legacy.sqlite'),base=path.join(dir,'gym.db'),restored=path.join(dir,'restored.db'),backup=path.join(dir,'backup.sqlcipher');
const db=new DatabaseSync(original);db.exec("CREATE TABLE clientes(id INTEGER PRIMARY KEY,nombre TEXT); INSERT INTO clientes VALUES(9,'Synthetic only');");db.close();
const source=fs.readFileSync(original),admin='Admin-synthetic-2026',password='Backup-synthetic-2026';
const results=[];
function run(operacion,base,respaldo,password_respaldo=password,password_admin=admin,expected=0){const r=cp.spawnSync(exe,['--mantenimiento'],{input:JSON.stringify({operacion,base,respaldo,password_respaldo,password_admin}),encoding:'utf8',windowsHide:true,timeout:30000});assert.equal(r.status,expected,operacion+': '+r.stderr);results.push({operacion,exitCode:r.status,status:'PASS'});}
run('importar_legacy',base,original);assert.deepEqual(fs.readFileSync(original),source);assert(!fs.readFileSync(base).subarray(0,16).equals(Buffer.from('SQLite format 3\0')));
run('respaldar',base,backup);
run('restaurar',restored,backup,'incorrect','incorrect',1);assert(!fs.existsSync(restored));
run('restaurar',restored,backup);
run('respaldar',restored,path.join(dir,'restored-backup.sqlcipher'));
run('restaurar',base,backup);
run('importar_legacy',base,original,password,admin,1);
fs.writeFileSync('.artifacts/production/maintenance-cli-results.json',JSON.stringify({results,temporaryDirectory:dir,realGymDataUsed:false},null,2));
console.log('PASS: release CLI import, encrypted backup, wrong credentials, restore, DPAPI across processes, no overwrite of existing DB; synthetic data only.');
