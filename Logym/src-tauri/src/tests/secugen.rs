const SG_DEV_FDU05: u32 = 0x06; // sgfplib.h: HU20 / Hamster Pro 20
    use super::*;
    #[test]
    fn apertura_59_libera_manager_y_reintento_recupera_sin_aperturas_simultaneas() {
        let _serial = TEST_SERIAL.lock().unwrap();
        *FAKE.lock().unwrap() = Fake { open_error: SGFDX_ERROR_DEV_ALREADY_OPEN, ..Fake::default() };
        let mut manager = None;
        for _ in 0..3 {
            let error = ensure_manager(&mut manager, &FAKE_API, 1).unwrap_err();
            for dato in ["SGFPM_OpenDevice: 59", "SGFDX_ERROR_DEV_ALREADY_OPEN", "modelo_enumerado=", "generacion_usb="] { assert!(error.contains(dato), "{error}"); }
            assert!(!error.contains("faltan drivers"));
            assert!(manager.is_none());
            assert_eq!(FAKE.lock().unwrap().active, 0);
        }
        assert_eq!(FAKE.lock().unwrap().terminated, 3);
        assert_eq!(FAKE.lock().unwrap().closed, 0, "No cerrar un dispositivo cuya apertura fue rechazada");
        FAKE.lock().unwrap().open_error = 0;
        ensure_manager(&mut manager, &FAKE_API, 1).unwrap();
        assert!(manager.as_ref().unwrap().connected);
        assert_eq!(FAKE.lock().unwrap().peak, 1);
        drop(manager);
        assert_eq!(FAKE.lock().unwrap().closed, 1);
        assert_eq!(FAKE.lock().unwrap().terminated, 4);
    }
    #[test]
    fn alta_rechaza_huella_duplicada_con_matcher_existente_incluso_inactiva() {
        let _serial=TEST_SERIAL.lock().unwrap();let sdk=SimulatedSdk::install();let reader=sdk.controller();
        let db=crate::test_support::db();
        let plan=crate::test_support::plan(&db,"Mensual");
        let id=crate::repositories::guardar_cliente(&db,crate::test_support::cliente(plan)).unwrap();
        db.execute("UPDATE clientes SET activo=0 WHERE id=?1",[id]).unwrap();
        let before=crate::test_support::counts(&db);
        let error=crate::repositories::guardar_cliente_sin_duplicar(&db,crate::test_support::cliente(plan),|a,b|reader.comparar_templates(a,b)).unwrap_err();
        assert!(error.contains("ya pertenece"));assert_eq!(crate::test_support::counts(&db),before);
        FAKE.lock().unwrap().match_error=106;
        assert!(crate::repositories::guardar_cliente_sin_duplicar(&db,crate::test_support::cliente(plan),|a,b|reader.comparar_templates(a,b)).is_err());
        assert_eq!(crate::test_support::counts(&db),before);
    }

    #[test]
    fn malformed_unicode_template_does_not_panic() {
        assert!(hex_decode("a€").is_err());
        assert!(hex_decode("zz").is_err());
        assert_eq!(hex_decode("0aFF").unwrap(), vec![10, 255]);
    }

    static TEST_SERIAL: Mutex<()> = Mutex::new(());
    #[derive(Default)]
    struct Fake {
        active: usize, peak: usize, terminated: usize, closed: usize,
        init_error: DWORD, open_error: DWORD, info_error: DWORD, close_error: DWORD,
        create_error: DWORD, null_create: bool, bad_dimensions: bool,
        enum_error: DWORD, enum_count: Option<DWORD>, null_devices: bool, other_model: bool,
        calls: Vec<&'static str>,
        image_error: DWORD, template_error: DWORD, match_error: DWORD,
    }
    static FAKE: Mutex<Fake> = Mutex::new(Fake {
        active: 0, peak: 0, terminated: 0, closed: 0, init_error: 0,
        open_error: 0, info_error: 0, close_error: 0, create_error: 0, null_create: false, bad_dimensions: false,
        enum_error: 0, enum_count: None, null_devices: false, other_model: false, calls: Vec::new(),
        image_error: 0, template_error: 0, match_error: 0,
    });
    unsafe extern "system" fn fake_create(out: *mut HSGFPM) -> DWORD {
        let mut fake = FAKE.lock().unwrap();
        fake.calls.push("create");
        if !fake.null_create {
            *out = Box::into_raw(Box::new(0u8)).cast();
            fake.active += 1;
            fake.peak = fake.peak.max(fake.active);
        }
        fake.create_error
    }
    unsafe extern "system" fn fake_terminate(handle: HSGFPM) -> DWORD {
        drop(Box::from_raw(handle.cast::<u8>()));
        let mut fake = FAKE.lock().unwrap();
        fake.active -= 1;
        fake.terminated += 1;
        fake.calls.push("terminate");
        0
    }
    unsafe extern "system" fn fake_init(_: HSGFPM, model: DWORD) -> DWORD {
        let mut fake = FAKE.lock().unwrap();
        fake.calls.push("init");
        assert_eq!(model, if fake.other_model { 8 } else { SG_DEV_FDU05 });
        fake.init_error
    }
    static DEVICE: DeviceList = DeviceList { dev_name: SG_DEV_FDU05, dev_id: 0, dev_type: 0, dev_sn: [0; 16] };
    static OTHER_DEVICE: DeviceList = DeviceList { dev_name: 8, dev_id: 0, dev_type: 0, dev_sn: [0; 16] };
    unsafe extern "system" fn fake_enum(_: HSGFPM, count: *mut DWORD, devices: *mut *mut DeviceList) -> DWORD {
        let mut fake = FAKE.lock().unwrap();
        fake.calls.push("enumerate");
        *count = fake.enum_count.unwrap_or(1);
        *devices = if fake.null_devices { std::ptr::null_mut() }
            else if fake.other_model { std::ptr::addr_of!(OTHER_DEVICE).cast_mut() }
            else { std::ptr::addr_of!(DEVICE).cast_mut() };
        fake.enum_error
    }
    unsafe extern "system" fn fake_format(_: HSGFPM, format: u16) -> DWORD {
        FAKE.lock().unwrap().calls.push("format");
        assert_eq!(format, SG_TEMPLATE_FORMAT_SG400);
        0
    }
    unsafe extern "system" fn fake_open(_: HSGFPM, port: DWORD) -> DWORD {
        assert_eq!(port, SG_USB_AUTO_DETECT);
        let mut fake = FAKE.lock().unwrap();
        fake.calls.push("open");
        fake.open_error
    }
    unsafe extern "system" fn fake_info(_: HSGFPM, out: *mut DeviceInfo) -> DWORD {
        let mut fake = FAKE.lock().unwrap();
        fake.calls.push("info");
        *out = DeviceInfo { image_width: if fake.bad_dimensions { 0 } else { 300 }, image_height: 400, ..DeviceInfo::default() };
        fake.info_error
    }
    unsafe extern "system" fn fake_close(_: HSGFPM) -> DWORD {
        let mut fake = FAKE.lock().unwrap();
        fake.closed += 1;
        fake.calls.push("close");
        fake.close_error
    }
    unsafe extern "system" fn fake_image(handle: HSGFPM, image: *mut u8, timeout: DWORD, _: *mut c_void, quality: DWORD) -> DWORD {
        assert!(!handle.is_null()); assert!(!image.is_null()); assert_eq!(timeout,4000); assert_eq!(quality,40);
        let mut fake=FAKE.lock().unwrap();fake.calls.push("image");fake.image_error
    }
    unsafe extern "system" fn fake_template(handle: HSGFPM, finger: *mut c_void, image: *mut u8, template: *mut u8) -> DWORD {
        assert!(!handle.is_null() && !finger.is_null() && !image.is_null() && !template.is_null());
        std::ptr::write_bytes(template,0xab,400);
        let mut fake=FAKE.lock().unwrap();fake.calls.push("template");fake.template_error
    }
    unsafe extern "system" fn fake_match(handle: HSGFPM, a: *mut u8, b: *mut u8, level: DWORD, result: *mut BOOL) -> DWORD {
        assert!(!handle.is_null()); assert_eq!(level,3);
        *result=(std::slice::from_raw_parts(a,400)==std::slice::from_raw_parts(b,400)) as BOOL;
        let mut fake=FAKE.lock().unwrap();fake.calls.push("match");fake.match_error
    }
    static FAKE_API: SgfpmApi = SgfpmApi {
        create: fake_create, terminate: fake_terminate, init: fake_init, enumerate_device: fake_enum,
        set_template_format: fake_format, open_device: fake_open,
        get_device_info: fake_info, close_device: fake_close,
        get_image_ex: fake_image, create_template: fake_template, match_template: fake_match,
    };

    // Instala exclusivamente punteros simulados. No carga DLL ni invoca USB.
    struct SimulatedSdk;
    impl SimulatedSdk {
        fn install() -> Self {
            STOPPING.store(false,Ordering::SeqCst);
            drop(GLOBAL_MANAGER.lock().unwrap().take());
            *FAKE.lock().unwrap()=Fake::default();
            API.get_or_init(||SgfpmApi { create:fake_create, terminate:fake_terminate, init:fake_init, enumerate_device:fake_enum,
                set_template_format:fake_format,open_device:fake_open,get_device_info:fake_info,get_image_ex:fake_image,
                create_template:fake_template,match_template:fake_match,close_device:fake_close });
            crate::lector_estado::tests::simular_usb(vec!["USB\\VID_1162&PID_2200\\simulado".into()]);
            Self
        }
        fn controller(&self) -> SecuGenController { SecuGenController::from_bin_path(Path::new("sdk-simulado")).unwrap() }
    }
    impl Drop for SimulatedSdk {
        fn drop(&mut self) {
            drop(GLOBAL_MANAGER.lock().unwrap_or_else(|e|e.into_inner()).take());
            STOPPING.store(false,Ordering::SeqCst);
            crate::lector_worker::pausar(false);
            assert_eq!(FAKE.lock().unwrap().active,0);
        }
    }

    #[test]
    fn simulated_capture_timeout_invalid_read_extraction_retry_and_exact_template_size() {
        let _serial=TEST_SERIAL.lock().unwrap();let sdk=SimulatedSdk::install();let reader=sdk.controller();
        FAKE.lock().unwrap().image_error=54;
        assert!(reader.capturar_template().unwrap().is_none());assert!(!GLOBAL_MANAGER.lock().unwrap().as_ref().unwrap().capture_verified);
        for code in [53,57,62] { FAKE.lock().unwrap().image_error=code;assert!(reader.capturar_template().unwrap_err().invalid_read()); }
        FAKE.lock().unwrap().image_error=0;FAKE.lock().unwrap().template_error=101;
        assert!(reader.capturar_template().unwrap_err().invalid_read());
        FAKE.lock().unwrap().template_error=0;let template=reader.capturar_template().unwrap().unwrap();
        assert_eq!(template,"ab".repeat(400));assert_eq!(hex_decode(&template).unwrap().len(),400);assert_eq!(FAKE.lock().unwrap().peak,1);
    }

    #[test]
    fn simulated_disconnect_recreates_manager_and_usb_generation_invalidates_old_handle() {
        let _serial=TEST_SERIAL.lock().unwrap();let sdk=SimulatedSdk::install();let reader=sdk.controller();
        FAKE.lock().unwrap().image_error=55;assert!(reader.capturar_template().unwrap_err().needs_recovery());
        assert!(reader.capturar_template().unwrap_err().needs_recovery());
        FAKE.lock().unwrap().image_error=0;let reader=sdk.controller();assert!(reader.capturar_template().unwrap().is_some());
        crate::lector_estado::tests::simular_usb(vec![]);assert!(reader.capturar_template().is_err());
        crate::lector_estado::tests::simular_usb(vec!["USB\\VID_1162&PID_2200\\simulado".into()]);
        assert!(reader.capturar_template().is_err());let reader=sdk.controller();assert!(reader.capturar_template().unwrap().is_some());
        assert_eq!(FAKE.lock().unwrap().terminated,2);assert_eq!(FAKE.lock().unwrap().peak,1);
    }

    #[test]
    fn simulated_matcher_contract_skips_inactive_and_malformed_stored_templates() {
        let _serial=TEST_SERIAL.lock().unwrap();let sdk=SimulatedSdk::install();let reader=sdk.controller();
        assert!(reader.comparar_templates(&"ab".repeat(400),&"ab".repeat(400)).unwrap());
        assert!(!reader.comparar_templates(&"ab".repeat(400),&"cd".repeat(400)).unwrap());
        for bad in ["a€".into(),"ab".repeat(399),"zz".repeat(400)] {assert!(reader.comparar_templates(&bad,&"ab".repeat(400)).is_err());}
        let db=crate::test_support::db();
        db.execute("INSERT INTO clientes(id,nombre,template_huella,activo) VALUES(1,'Inactivo',?1,0),(2,'Malformado','a€',1),(3,'Coincidente',?1,1)",["ab".repeat(400)]).unwrap();
        assert_eq!(reader.identificar_template(&db,&"ab".repeat(400)).unwrap(),Some(3));
        FAKE.lock().unwrap().match_error=106;
        assert!(matches!(reader.identificar_template(&db,&"ab".repeat(400)),Err(SecuGenError::Sdk{function:"SGFPM_MatchTemplate",code:106})));
    }

    #[test]
    fn simulated_pause_blocks_initialization_and_shutdown_rejects_new_native_work() {
        let _serial=TEST_SERIAL.lock().unwrap();let sdk=SimulatedSdk::install();
        crate::lector_worker::pausar(true);
        assert!(crate::lector_worker::inicializar_lector(Path::new("sdk-simulado")).unwrap().is_none());
        assert_eq!(FAKE.lock().unwrap().active,0);
        crate::lector_worker::pausar(false);let reader=sdk.controller();
        request_shutdown();assert!(reader.capturar_template().is_err());assert!(SecuGenController::from_bin_path(Path::new("sdk-simulado")).is_err());
        shutdown();assert_eq!(FAKE.lock().unwrap().active,0);assert_eq!(FAKE.lock().unwrap().terminated,1);
    }

    #[test]
    fn simulated_concurrent_controllers_serialize_sdk_and_share_one_manager() {
        let _serial=TEST_SERIAL.lock().unwrap();let sdk=SimulatedSdk::install();let _reader=sdk.controller();
        let threads:Vec<_>=(0..6).map(|_|std::thread::spawn(||{
            let reader=SecuGenController::from_bin_path(Path::new("sdk-simulado")).unwrap();
            assert_eq!(reader.capturar_template().unwrap().unwrap().len(),800);
        })).collect();
        for thread in threads {thread.join().unwrap();}
        assert_eq!(FAKE.lock().unwrap().peak,1);assert_eq!(FAKE.lock().unwrap().calls.iter().filter(|s|**s=="image").count(),6);
    }

    #[test]
    fn official_sequence_uses_enumerated_model_and_timeout_does_not_prove_capture() {
        let _serial = TEST_SERIAL.lock().unwrap();
        *FAKE.lock().unwrap() = Fake { other_model: true, ..Fake::default() };
        let mut manager = create_manager(&FAKE_API, 1).unwrap();
        assert_eq!(FAKE.lock().unwrap().calls, ["create", "enumerate", "init", "format", "open", "info"]);
        for code in [54, 53, 57, 62, 54] {
            manager.record_capture(code);
            assert!(!manager.capture_verified);
        }
        manager.record_capture(0);
        assert!(manager.capture_verified);
        manager.record_capture(54);
        assert!(manager.capture_verified); // Solo conserva evidencia previa de este manager.
        manager.record_capture(2);
        assert!(!manager.capture_verified);
        manager.record_capture(54);
        assert!(!manager.capture_verified);
        drop(manager);
        let fake = FAKE.lock().unwrap();
        assert_eq!(fake.active, 0);
        assert_eq!(&fake.calls[fake.calls.len()-2..], ["close", "terminate"]);
    }

    #[test]
    fn failed_empty_null_or_ambiguous_enumeration_is_released_before_init() {
        let _serial = TEST_SERIAL.lock().unwrap();
        for fake in [
            Fake { enum_error: 6, ..Fake::default() },
            Fake { enum_count: Some(0), ..Fake::default() },
            Fake { null_devices: true, ..Fake::default() },
            Fake { enum_count: Some(2), ..Fake::default() },
        ] {
            *FAKE.lock().unwrap() = fake;
            assert!(create_manager(&FAKE_API, 1).is_err());
            let fake = FAKE.lock().unwrap();
            assert_eq!(fake.calls, ["create", "enumerate", "terminate"]);
            assert_eq!(fake.active, 0);
            assert_eq!(fake.terminated, 1);
            assert_eq!(fake.closed, 0);
        }
    }

    #[test]
    fn repeated_init_6_is_released_and_next_attempt_can_recover() {
        let _serial = TEST_SERIAL.lock().unwrap();
        *FAKE.lock().unwrap() = Fake { init_error: 6, ..Fake::default() };
        let mut manager = None;
        for _ in 0..3 {
            assert!(ensure_manager(&mut manager, &FAKE_API, 1).unwrap_err().contains("SGFPM_Init: 6"));
            assert!(manager.is_none());
            assert_eq!(FAKE.lock().unwrap().active, 0);
        }
        FAKE.lock().unwrap().init_error = 0;
        ensure_manager(&mut manager, &FAKE_API, 1).unwrap();
        FAKE.lock().unwrap().close_error = 55;
        ensure_manager(&mut manager, &FAKE_API, 2).unwrap();
        assert_eq!(manager.as_ref().unwrap().usb_generation, 2);
        assert_eq!(FAKE.lock().unwrap().peak, 1);
        drop(manager);
        let fake = FAKE.lock().unwrap();
        assert_eq!(fake.active, 0);
        assert_eq!(fake.terminated, 5);
        assert_eq!(fake.closed, 2);
    }

    #[test]
    fn open_info_and_dimension_failures_release_native_resources() {
        let _serial = TEST_SERIAL.lock().unwrap();
        for (open_error, info_error, bad_dimensions, expected_closes) in [
            (55, 0, false, 0), (0, 2, false, 1), (0, 0, true, 1),
        ] {
            *FAKE.lock().unwrap() = Fake { open_error, info_error, bad_dimensions, ..Fake::default() };
            assert!(create_manager(&FAKE_API, 1).is_err());
            let fake = FAKE.lock().unwrap();
            assert_eq!(fake.active, 0);
            assert_eq!(fake.terminated, 1);
            assert_eq!(fake.closed, expected_closes);
        }
    }

    #[test]
    fn null_or_failed_create_never_leaks_a_returned_handle() {
        let _serial = TEST_SERIAL.lock().unwrap();
        *FAKE.lock().unwrap() = Fake { null_create: true, ..Fake::default() };
        assert!(create_manager(&FAKE_API, 1).is_err());
        assert_eq!(FAKE.lock().unwrap().terminated, 0);
        *FAKE.lock().unwrap() = Fake { create_error: 1, ..Fake::default() };
        assert!(create_manager(&FAKE_API, 1).is_err());
        assert_eq!(FAKE.lock().unwrap().terminated, 1);
        assert_eq!(FAKE.lock().unwrap().active, 0);
    }

    #[test]
    fn transient_capture_errors_do_not_reconnect() {
        for code in [53, 54, 57, 62] {
            let error = SecuGenError::Sdk { function: "SGFPM_GetImageEx", code };
            assert!(!error.needs_recovery(), "code {code}");
            assert_eq!(error.invalid_read(), code != 54);
        }
    }

    #[test]
    fn hu20_device_constants_match_official_header() {
        assert_eq!(SG_DEV_FDU05, 6);
        assert_eq!(SG_USB_AUTO_DETECT, 957);
        assert_eq!(SG_TEMPLATE_FORMAT_SG400, 0x0200);
        assert_eq!(SG_TEMPLATE_SIZE, 400);
        assert_eq!(SG_SECURITY_LEVEL_NORMAL, 3);
    }

    #[test]
    fn extraction_errors_allow_fast_retry() {
        for code in [2, 101, 105] {
            let error = SecuGenError::Sdk { function: "SGFPM_CreateTemplate", code };
            assert!(error.invalid_read());
            assert!(!error.needs_recovery());
            assert!(!error.persistent_fault());
        }
    }

    #[test]
    fn connection_failures_require_recovery() {
        for code in [51, 52, 55, 56, 58, 59, 60, 63] {
            let error = SecuGenError::Sdk { function: "SGFPM_GetImageEx", code };
            assert!(error.needs_recovery(), "code {code}");
            assert!(!error.invalid_read());
        }
        assert!(SecuGenError::Disconnected.needs_recovery());
        assert!(SecuGenError::Initialization("open failed".into()).needs_recovery());
        assert!(!SecuGenError::Other("database failed".into()).needs_recovery());
    }

    #[test]
    fn persistent_sdk_faults_do_not_imply_usb_disconnection() {
        for code in [1, 2, 3, 5, 8, 61, 501, 502, 503, 504, 999] {
            let error = SecuGenError::Sdk { function: "SGFPM_GetImageEx", code };
            assert!(!error.needs_recovery(), "code {code}");
            assert!(error.persistent_fault(), "code {code}");
        }
        for code in [3, 501, 502, 503, 504, 999] {
            let error = SecuGenError::Sdk { function: "SGFPM_CreateTemplate", code };
            assert!(!error.invalid_read());
            assert!(error.persistent_fault());
        }
        for code in [53, 54, 57, 62] {
            assert!(!SecuGenError::Sdk { function: "SGFPM_GetImageEx", code }.persistent_fault());
        }
    }

#[test]
fn sqlite_bloqueado_no_retiene_sdk_y_no_invierte_orden() {
    use std::sync::{Arc,atomic::{AtomicBool,Ordering},mpsc};use std::time::{Duration,Instant};
    static BLOQUEADO:AtomicBool=AtomicBool::new(false);
    fn ocupado(n:i32)->bool {BLOQUEADO.store(true,Ordering::SeqCst);std::thread::sleep(Duration::from_millis(5));n<200}
    let _serial=TEST_SERIAL.lock().unwrap();let sdk=SimulatedSdk::install();let reader=Arc::new(sdk.controller());
    let temp=crate::test_support::TempDb::new();let db=Connection::open(&temp.path).unwrap();crate::database::inicializar_esquema(&db).unwrap();
    let id=crate::test_support::vigente(&db);db.execute_batch("PRAGMA journal_mode=DELETE; BEGIN EXCLUSIVE;").unwrap();BLOQUEADO.store(false,Ordering::SeqCst);
    let path=temp.path.clone();let r=reader.clone();let identify=std::thread::spawn(move||{let db=Connection::open(path).unwrap();db.busy_handler(Some(ocupado)).unwrap();r.identificar_template(&db,&"ab".repeat(400))});
    let start=Instant::now();while !BLOQUEADO.load(Ordering::SeqCst)&&start.elapsed()<Duration::from_secs(2){std::thread::sleep(Duration::from_millis(2));}
    let observed=BLOQUEADO.load(Ordering::SeqCst);let(rtx,rrx)=mpsc::channel();let r=reader.clone();let compare=std::thread::spawn(move||{rtx.send(r.comparar_templates(&"ab".repeat(400),&"ab".repeat(400))).unwrap();});
    let available=rrx.recv_timeout(Duration::from_millis(500));db.execute_batch("COMMIT").unwrap();let found=identify.join().unwrap();compare.join().unwrap();
    assert!(observed,"No se provoco la espera SQLite");assert!(available.unwrap().unwrap(),"El SDK quedo retenido esperando SQLite");assert_eq!(found.unwrap(),Some(id));
}

#[test]
fn altas_y_checkins_simultaneos_con_sdk_real_simulado_sin_cobros_duplicados() {
    use std::sync::{Arc,Barrier,mpsc};use std::time::Duration;
    let _serial=TEST_SERIAL.lock().unwrap();let sdk=SimulatedSdk::install();let reader=Arc::new(sdk.controller());
    let temp=crate::test_support::TempDb::new();let db=temp.connect();crate::database::inicializar_esquema(&db).unwrap();let id=crate::test_support::vigente(&db);let plan=crate::test_support::plan(&db,"Mensual");drop(db);
    let barrier=Arc::new(Barrier::new(8));let(tx,rx)=mpsc::channel();let mut threads=Vec::new();
    for i in 0..8 {let path=temp.path.clone();let r=reader.clone();let b=barrier.clone();let tx=tx.clone();threads.push(std::thread::spawn(move||{
        let db=Connection::open(path).unwrap();db.busy_timeout(Duration::from_secs(3)).unwrap();b.wait();
        let result=if i<4 {let mut datos=crate::test_support::cliente(plan);datos.template_huella="cd".repeat(400);crate::repositories::guardar_cliente_sin_duplicar(&db,datos,|a,b|r.comparar_templates(a,b)).map(|_|"alta".to_string())}
            else {r.identificar_template(&db,&"ab".repeat(400)).map_err(String::from).and_then(|found|{assert_eq!(found,Some(id));crate::repositories::registrar_asistencia(&db,id)})};
        tx.send((i,result)).unwrap();}));}
    drop(tx);let mut altas=0;let mut asistencias=0;for _ in 0..8 {let(i,result)=rx.recv_timeout(Duration::from_secs(10)).expect("Timeout: posible deadlock");if i<4 {match result {Ok(_)=>altas+=1,Err(e)=>assert!(e.contains("ya pertenece"),"{e}")}}else {match result {Ok(_)=>asistencias+=1,Err(e)=>assert!(e.contains("Ya registr"),"{e}")}}}
    for t in threads{t.join().unwrap();}assert_eq!(altas,1);assert_eq!(asistencias,1);assert_eq!(crate::test_support::counts(&temp.connect()),(2,2,2,1));
}

#[test]
fn alta_corrupta_sdk_error_y_duplicado_no_dejan_registros_parciales() {
    let _serial=TEST_SERIAL.lock().unwrap();let sdk=SimulatedSdk::install();let reader=sdk.controller();let db=crate::test_support::db();let plan=crate::test_support::plan(&db,"Mensual");
    let id=crate::repositories::guardar_cliente_sin_duplicar(&db,crate::test_support::cliente(plan),|a,b|reader.comparar_templates(a,b)).unwrap();
    for activo in [1,0] {db.execute("UPDATE clientes SET activo=?1 WHERE id=?2",rusqlite::params![activo,id]).unwrap();let before=crate::test_support::counts(&db);assert!(crate::repositories::guardar_cliente_sin_duplicar(&db,crate::test_support::cliente(plan),|a,b|reader.comparar_templates(a,b)).unwrap_err().contains("ya pertenece"));assert_eq!(crate::test_support::counts(&db),before);}
    FAKE.lock().unwrap().match_error=106;let before=crate::test_support::counts(&db);assert!(crate::repositories::guardar_cliente_sin_duplicar(&db,crate::test_support::cliente(plan),|a,b|reader.comparar_templates(a,b)).is_err());assert_eq!(crate::test_support::counts(&db),before);
    FAKE.lock().unwrap().match_error=0;db.execute("UPDATE clientes SET template_huella='corrupta' WHERE id=?1",[id]).unwrap();assert!(crate::repositories::guardar_cliente_sin_duplicar(&db,crate::test_support::cliente(plan),|a,b|reader.comparar_templates(a,b)).unwrap_err().contains("dañada"));assert_eq!(crate::test_support::counts(&db),before);
}

#[test]
fn clientes_sin_huella_no_interfieren_con_identificacion_ni_duplicidad() {
    let _serial = TEST_SERIAL.lock().unwrap();
    let sdk = SimulatedSdk::install();
    let reader = sdk.controller();
    let db = crate::test_support::db();
    let plan = crate::test_support::plan(&db, "Mensual");
    let mut datos = crate::test_support::cliente(plan);
    datos.sin_huella = true;
    datos.template_huella.clear();
    crate::repositories::guardar_cliente_sin_duplicar(&db, datos, |_,_| panic!("Alta sin SDK")).unwrap();
    let id = crate::repositories::guardar_cliente_sin_duplicar(&db, crate::test_support::cliente(plan), |a,b| reader.comparar_templates(a,b)).unwrap();
    db.execute("UPDATE clientes SET fecha_inicio_membresia=date('now','localtime'),fecha_fin_membresia=date('now','localtime','+30 days') WHERE id=?1", [id]).unwrap();
    // Primero manual, después identificación biométrica: una sola asistencia.
    crate::repositories::registrar_asistencia(&db, id).unwrap();
    assert_eq!(reader.identificar_template(&db, &"ab".repeat(400)).unwrap(), Some(id));
    assert!(crate::repositories::registrar_asistencia(&db, id).unwrap_err().contains("Ya registró"));
    assert!(crate::repositories::guardar_cliente_sin_duplicar(&db, crate::test_support::cliente(plan), |a,b| reader.comparar_templates(a,b)).unwrap_err().contains("ya pertenece"));
    assert_eq!(crate::test_support::counts(&db), (2,2,2,1));
}
