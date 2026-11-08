    use super::*;
    #[test]
    fn historical_revenue_ignores_catalog_price_changes() {
        let db = Connection::open_in_memory().unwrap();
        db.execute_batch("
            CREATE TABLE pagos (
                total_centavos INTEGER NOT NULL,
                membresia_centavos INTEGER NOT NULL,
                inscripcion_centavos INTEGER NOT NULL,
                fecha_pago TEXT NOT NULL
            );
            CREATE TABLE tipos_membresia (id INTEGER PRIMARY KEY, precio_centavos INTEGER);
            INSERT INTO tipos_membresia VALUES(1,35000);
            INSERT INTO pagos VALUES(50000,35000,15000,datetime('now','localtime'));
            UPDATE tipos_membresia SET precio_centavos=40000 WHERE id=1;
        ").unwrap();
        let reporte = ingresos(&db,"mes").unwrap();
        assert_eq!(reporte.total_centavos,50000);
        assert_eq!(reporte.membresias_centavos,35000);
        assert_eq!(reporte.inscripciones_centavos,15000);
    }
    #[test]
    fn rejects_invalid_period() {
        let db = Connection::open_in_memory().unwrap();
        assert!(ingresos(&db,"invalido").is_err());
    }
