    use super::*;
    #[test]
    fn migracion_conserva_clientes_y_asistencias() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("
            CREATE TABLE clientes(id INTEGER PRIMARY KEY,nombre TEXT,telefono TEXT,fecha_registro TEXT,fecha_inicio_membresia TEXT,fecha_fin_membresia TEXT);
            CREATE TABLE asistencias(id INTEGER PRIMARY KEY,cliente_id INTEGER,fecha_hora TEXT);
            INSERT INTO clientes(id,nombre) VALUES(7,'Cliente antiguo');
            INSERT INTO asistencias(id,cliente_id) VALUES(9,7);
        ").unwrap();
        inicializar_esquema(&conn).unwrap();
        inicializar_esquema(&conn).unwrap();
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM clientes WHERE id=7",[],|r|r.get::<_,i64>(0)).unwrap(),1);
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM asistencias WHERE id=9",[],|r|r.get::<_,i64>(0)).unwrap(),1);
        assert_eq!(conn.query_row("SELECT COUNT(*) FROM tipos_membresia",[],|r|r.get::<_,i64>(0)).unwrap(),5);
    }
    #[test]
    fn inscripcion_gratis_no_se_restaurara_al_reiniciar() {
        let conn = Connection::open_in_memory().unwrap();
        inicializar_esquema(&conn).unwrap();
        conn.execute("UPDATE configuracion_cobros SET inscripcion_centavos=0",[]).unwrap();
        inicializar_esquema(&conn).unwrap();
        assert_eq!(conn.query_row("SELECT inscripcion_centavos FROM configuracion_cobros WHERE id=1",[],|r|r.get::<_,i64>(0)).unwrap(),0);
    }
