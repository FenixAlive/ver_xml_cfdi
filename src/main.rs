//#![windows_subsystem = "windows"]
use web_view::*;
use tinyfiledialogs as tfd;
use std::{env, fs, path::Path, collections::HashMap};
use xmltree::Element;
use format_money::format_money;
use ureq;

//TODO: hacer complemento de pagos, nomina y traslado, complemento de predial y aduanero
//cambiar el titulo final webview, descomentar primera linea y debug(false)
fn main() {
    // webview
    let html = format!(include_str!("./app/app.html"),
                css = include_str!("./app/styles.css"),
                qrcode = include_str!("./app/qrcode.min.js"),
                javascript = include_str!("./app/app.js")
                );
    web_view::builder()
    .title("Cheetah: Visor de CFDI v.3.3 desde su archivo xml")
    .content(Content::Html(html))
    .size(590, 570)
    .resizable(true)
    .debug(true)
    .user_data(())
    .invoke_handler(|webview, arg| {
        if arg == "inicio" {
            if let Some(cfdi) = abrir_xml(){
                webview.eval("mostrarApp(true)")?;
                let datos_val = datos_cfdi(webview, &cfdi)?;
                //validar xml
                validar_cfdi_sat(webview, &datos_val.0, &datos_val.1, &datos_val.2, &datos_val.3)?;
            } else {
                webview.exit();
            }
        }else{
            println!("desde js: {}", arg);
        }
        Ok(())
    })
    .run()
    .unwrap();
}

fn abrir_xml()-> Option<Element> {
    let args: Vec<String> = env::args().collect();
    let mut open_file: String = String::new();
    if args.len() == 2 {
        open_file = args[1].to_owned();
    }else{
        let mut buscar = tfd::YesNo::Yes;
        while buscar == tfd::YesNo::Yes { 
            match tfd::open_file_dialog("Elige XML de CFDI que quieres ver", "", None){
                Some(file) => {
                    open_file = file;
                    buscar = tfd::YesNo::No;
                },
                None => {    
                    buscar = tfd::message_box_yes_no("No elegiste ningun archivo", "¿Quieres buscar de nuevo?", tfd::MessageBoxIcon::Question, tfd::YesNo::No);
                    if buscar == tfd::YesNo::No{
                        return None;
                    }
                }
            }
        }
    }
    return leer_cfdi(Path::new(&open_file))
}

//función que implementa el leer un cfdi y validarlo de forma simple
fn leer_cfdi(path: &Path) -> Option<Element>{
    if path.extension() == Some(std::ffi::OsStr::new("xml")){
        let xml = match fs::read_to_string(path) {
            Ok(xm) => xm,
            Err(e) => {
                tfd::message_box_ok("Error", "Error al leer el archivo", tfd::MessageBoxIcon::Error);
                eprintln!("Error en la extension {}", e);
                return None;
            }
        };
        let xml : Vec<&str> = xml.split("\u{feff}").collect(); 
        let xml_final: &str;
        if xml.len() > 1{
            xml_final = xml[1];
        }else{
            xml_final = xml[0];
        }
        match Element::parse(xml_final.as_bytes()) {
            Ok(cfdi) => {
                if cfdi.name != "Comprobante" || get_data(&cfdi, "Version") != "3.3"{
                    tfd::message_box_ok("Error", "Este xml no es un comprobante CFDI v.3.3 valido", tfd::MessageBoxIcon::Error);
                    eprintln!("Comprobante no valido");
                    return None
                }else {
                    return Some(cfdi)
                }
            },
            Err(e) => {
                tfd::message_box_ok("Error", "Comprobante no valido", tfd::MessageBoxIcon::Error);
                eprintln!("Comprobante no valido {}", e);
                }
        }
    }else{
        tfd::message_box_ok("Error", "Extensión del archivo no valida", tfd::MessageBoxIcon::Error);
        eprintln!("Extensión del archivo no valida");
    }
    return None
}

//función que lea datos del cfdi y vaya mandandolas a funciones js para llenar
fn datos_cfdi(web: &mut web_view::WebView<'_, ()>, cfdi: & Element) -> Result<(String, String, String, String), web_view::Error> {
    let mut uuid: &str = "";
    let mut rfc_emi: &str = "";
    let mut rfc_rec: &str = "";
    let total = get_data(&cfdi, "Total");
    mandar_datos_web_view_cabe(web, cfdi, "Serie", "serie", "Serie")?;
    mandar_datos_web_view_cabe(web, cfdi, "Folio", "folio", "Folio")?;
    let fecha = get_data(&cfdi, "Fecha");
    let mut fecha_c = String::from("");
    if fecha != ""{
        let fecha_t = separar_datetime(fecha);
        fecha_c = format!("{} a las {} hrs", fecha_t.0, fecha_t.1);
    }
    web.eval(&format!("rellenarCabe('{}', '{}', '{}')", "fecha", "Fecha",fecha_c))?;
    tipo_cfdi(web, cfdi)?;
    mandar_datos_web_view_cabe(web, cfdi, "LugarExpedicion", "lugarExp", "C.P. Expedición")?;
    mandar_datos_web_view(web, cfdi, "Version", "versionCfdi", "Versión CFDI")?;
    let metodo_p = metodo_pago(cfdi, "MetodoPago")?;
    web.eval(&format!("rellenar('metodoPago', 'Metodo de Pago', '{}')", metodo_p))?;
    let forma_p = forma_pago(cfdi, "FormaPago")?;
    web.eval(&format!("rellenar('formaPago', 'Forma de Pago', '{}')", forma_p))?;
    let subtotal = get_data(&cfdi, "SubTotal");
    web.eval(&format!("rellenar('{}', '{}', '{}')", "subtotal", "Subtotal", format_money(subtotal)))?;
    //si hay descuento mostrar linea de neto subtotal
    let descuento = get_data(&cfdi, "Descuento");
    if descuento != ""{
        let sub_num = match subtotal.parse::<f64>() {
            Ok(val) => val,
            Err(_) => 0.0,
        };
        let des_num = match descuento.parse::<f64>() {
            Ok(val) => val,
            Err(_) => 0.0,
        };
        let suma_sub = &format!("{:.2}",(sub_num-des_num));
        web.eval(&format!("rellenar('{}', '{}', '- {}')", "descuento", "Descuento", format_money(descuento)))?;
        web.eval(&format!("rellenar('{}', '{}', '{}')", "subNeto", "Subtotal Neto", format_money(suma_sub)))?;
    }
    web.eval(&format!("rellenar('{}', '{}', '{}')", "total", "Total", format_money(total)))?;
    mandar_datos_web_view(web, cfdi, "CondicionesDePago", "condicionesPago", "Condiciones de Pago")?;
    mandar_datos_web_view(web, cfdi, "Moneda", "moneda", "Moneda")?;
    mandar_datos_web_view(web, cfdi, "NoCertificado", "certEmi", "Numero de Certificado Emisór")?;
    //si el tipo de cambio existe y es distinto de 1 hacer calculo  de total en pesos y mandarlo
    let tipo_cambio = get_data(&cfdi, "TipoCambio");
    if tipo_cambio != "1" && tipo_cambio != ""{
        let tipo_cambio_num = match tipo_cambio.parse::<f64>() {
            Ok(val) => val,
            Err(_) => 0.0,
        };
        let total_num = match total.parse::<f64>() {
            Ok(val) => val,
            Err(_) => 0.0,
        };
        let total_pesos = format!("{}",total_num * tipo_cambio_num);
        web.eval(&format!("rellenar('{}', '{}', '{}')", "tipoCambio", "Tipo de Cambio", format_money(tipo_cambio)))?;
        web.eval(&format!("rellenar('{}', '{}', '{}')", "totalPesos", "Total en Pesos", format_money(&total_pesos)))?;
    }
    //iterar cfdi
    for cf in cfdi.children.iter(){
        match cf.name.as_ref() {
            "Emisor" => {
                rfc_emi = get_data(cf, "Rfc");
                web.eval(&format!("rellenar('{}', '{}', '{}')", "rfcEmi", "RFC Emisor",rfc_emi))?;
                mandar_datos_web_view(web, cf, "Nombre", "razonEmi", "Razón Social")?;
                regimen_fiscal(web, cf)?;
            },
            "Receptor" => {
                rfc_rec = get_data(cf, "Rfc");
                web.eval(&format!("rellenar('{}', '{}', '{}')", "rfcRec", "RFC Receptor",rfc_rec))?;
                mandar_datos_web_view(web, cf, "Nombre", "razonRec", "Razón Social")?;
                uso_cfdi(web, cf)?;
            },
            "Conceptos" => {
                web.eval("showConcep(true)")?;
                let mut idx_concep = 0;
                for concep in cf.children.iter() {
                    web.eval(&format!("addConcep(`{}`, `{}`, `{}`, `{}`, `{}`, `{}`, `{}`, `{}`, `{}`, `- {}`)", 
                            idx_concep, get_data(&concep, "Cantidad"), get_data(&concep, "ClaveUnidad"), get_data(&concep, "Unidad"),
                            get_data(&concep, "ClaveProdServ"), get_data(&concep, "NoIdentificacion"), format_money(get_data(&concep, "ValorUnitario")), 
                            get_data(&concep, "Descripcion"), format_money(get_data(&concep, "Importe")), format_money(get_data(&concep, "Descuento"))))?;
                    for impu_concep in concep.children.iter(){
                        match impu_concep.name.as_ref(){
                            "Impuestos" =>{
                                let mut id_imp = 0;
                                web.eval(&format!("trasRetConcepCabe(`{}`)", idx_concep))?;
                                for tras_ret in impu_concep.children.iter(){
                                    for tr_re in tras_ret.children.iter(){
                                        web.eval(&format!("addTrasRetConcep('{}', `{}`, `{}`, `{}`, `{}`, `{}`, `{}`, `{}`)", 
                                                idx_concep, id_imp, tras_ret.name, format_money(get_data(&tr_re, "Importe")), get_data(&tr_re, "TasaOCuota"),
                                                get_data(&tr_re, "TipoFactor"), completar_impuesto(get_data(&tr_re, "Impuesto")), format_money(get_data(&tr_re, "Base"))))?;
                                        id_imp += 1;
                                    }
                                }
                            },
                            _ => println!("falta en concepto: {:?}", impu_concep.name)
                        }
                    }
                    idx_concep += 1;
                }
            }, 
            "Impuestos" => {
                let mut idx_imp = 0;
                web.eval("impuestosCabe()")?;
                for tras_ret in cf.children.iter() {
                    for tr_re in tras_ret.children.iter(){
                        web.eval(&format!("addTrasRetImp(`{}`, `{}`, `{}`, `{}`, `{}`, `{}`)", 
                                            idx_imp, tr_re.name, completar_impuesto(get_data(&tr_re, "Impuesto")), 
                                            get_data(&tr_re, "TipoFactor"), get_data(&tr_re, "TasaOCuota"), format_money(get_data(&tr_re, "Importe"))))?;
                        idx_imp += 1;
                    }
                }
            },
            "Complemento" => {
                for com in cf.children.iter() {
                    match com.name.as_ref(){
                        "TimbreFiscalDigital" => {
                            let fecha_tim = get_data(com, "FechaTimbrado");
                            let mut fecha_c = String::from("");
                            if fecha_tim != "" {
                                let fecha_tem = separar_datetime(fecha_tim);
                                fecha_c = format!("{} a las {} hrs", fecha_tem.0, fecha_tem.1);
                            }
                            web.eval(&format!("rellenar('{}', '{}', '{}')", "fechaTimbre", "Fecha de Timbrado",fecha_c))?;
                            mandar_datos_web_view(web, com, "NoCertificadoSAT", "certSat", "Numero de Certificado SAT")?;
                            web.eval(&format!("rellenar_cortado('{}', '{}', '{}')", "selloSat", "Sello Digital del SAT", get_data(&com, "SelloSAT")))?;
                            uuid = get_data(&com, "UUID");
                            web.eval(&format!("rellenarCabe('{}', '{}', '{}')", "uuid", "Folio Fiscal UUID",uuid))?;
                            let mut leyenda = String::from(get_data(&com, "Leyenda"));
                            if leyenda != ""{
                                leyenda = format!("|{}", leyenda);
                            }
                            let cadena = format!("||{}|{}|{}|{}{}|{}|{}||", get_data(&com, "Version"), 
                                                uuid, fecha_tim, get_data(&com, "RfcProvCertif"), leyenda, 
                                                get_data(&com, "SelloCFD"), get_data(&com, "NoCertificadoSAT"));
                            web.eval(&format!("rellenar_cortado('{}', '{}', '{}')", "cadenaTim", "Cadena Original del Timbre",cadena))?;
                        },
                        "Pagos" => {
                            let mut id = 0;
                            web.eval("pagosCont()")?;
                            for pago in com.children.iter() {
                                let fecha_pago = get_data(pago, "FechaPago");
                                let mut fecha_c = String::from("");
                                if fecha_pago != ""{
                                    let fecha_t = separar_datetime(fecha_pago);
                                    fecha_c = format!("{} a las {} hrs", fecha_t.0, fecha_t.1);
                                }
                                let forma_p = forma_pago(pago, "FormaDePagoP")?;
                                web.eval(&format!("pagoCabe('{}','{}','{}','{}','{}','{{}}')", id, fecha_c, forma_p, get_data(pago, "MonedaP"), format_money(get_data(pago, "Monto"))))?;
                                id += 1;   
                            }
                        },
                        _ => println!("falta en Complemento: {:?}", com.name)
                    }
                }
            },
            "CfdiRelacionados" => {
                tipo_relacion(web, cf)?;
                let mut idx_rel = 0;
                for rel in cf.children.iter() {
                    web.eval(&format!("addRelacionado(`{}`, `{}`)", idx_rel, get_data(&rel, "UUID")))?;
                    idx_rel += 1;
                }
            }
            _ => println!("falta en cfdi: {:?}", cf.name)
        }
    }
    let sello = get_data(&cfdi, "Sello");
    web.eval(&format!("rellenar_cortado('{}', '{}', '{}')", "selloEmi", "Sello Digital del CFDI", sello))?;
    //imprimir qr
    web.eval(&format!("ponerQr('https://verificacfdi.facturaelectronica.sat.gob.mx/default.aspx?id={}&re={}&rr={}&tt={}&fe={}')", 
                        uuid, rfc_emi, rfc_rec, total, &sello[sello.len()-8..]))?;
    Ok((rfc_emi.to_string(), rfc_rec.to_string(), total.to_string(), uuid.to_string()))
}

//envia los datos primarios del cfdi al webview
fn mandar_datos_web_view(web: &mut web_view::WebView<'_, ()>, cfdi: & Element, at_xml: &str, id_html: &str, tit_htlm: &str) ->  Result<String, web_view::Error>{
    let dato =  get_data(&cfdi, at_xml);
    web.eval(&format!("rellenar('{}', '{}', '{}')", id_html, tit_htlm, dato))?;
    Ok(dato.to_string())
}

//ver si puedo juntar las dos para cabecera y rellenar normal
fn mandar_datos_web_view_cabe(web: &mut web_view::WebView<'_, ()>, cfdi: & Element, at_xml: &str, id_html: &str, tit_htlm: &str) ->  Result<String, web_view::Error>{
    let dato =  get_data(&cfdi, at_xml);
    web.eval(&format!("rellenarCabe('{}', '{}', '{}')", id_html, tit_htlm, dato))?;
    Ok(dato.to_string())
}

//función que me ayuda a leer los elementos del cfdi mas rapido
fn get_data<'a>(cfdi: &'a Element, key: &str) -> &'a str {
    match cfdi.attributes.get(key) {
        Some(d) => {
            //println!("{}: {}", key, d);
            return d;
            },
        None => {
            //println!("{} no esta en get_data", key);
            return "";
        }
    };
}

fn separar_datetime(datetime: &str) -> (String, String){
    let dt_sp: Vec<&str> = datetime.split("T").collect();
    (dt_sp[0].to_string(), dt_sp[1].to_string())
}

//completa el tipo de impuesto
fn completar_impuesto(imp: &str) -> String{
    let val;
    match imp{
        "001"=>{val = "ISR";},
        "002"=>{val = "IVA";},
        "003"=>{val = "IEPS";},
        _=>{return imp.to_string()}
    }
    format!("{}-{}",imp, val)
}

fn tipo_cfdi(web: &mut web_view::WebView<'_, ()>, cfdi: & Element) -> Result<(), web_view::Error>{
    let tipo_hash: HashMap<&str, &str> = [("I", "Ingreso"), ("E", "Egreso"), ("T", "Traslado"), ("N", "Nómina"), ("P", "Pago")].iter().cloned().collect();
    let key =  get_data(&cfdi, "TipoDeComprobante");
    if let Some(tipo) = tipo_hash.get(key){
        web.eval(&format!("rellenarCabe('{}', '{}', '{}')", "tipoComp", "Tipo Comprobante", format!("{} - {}",key,tipo)))?;
    }
    Ok(())
}

fn tipo_relacion(web: &mut web_view::WebView<'_, ()>, cfdi: & Element) -> Result<(), web_view::Error>{
    let tipo_rel: HashMap<&str, &str> = [("01","Nota de crédito de los documentos relacionados"),("02","Nota de débito de los documentos relacionados"),
                                                ("03","Devolución de mercancía sobre facturas o traslados previos"),("04","Sustitución de los CFDI previos"),
                                                ("05","Traslados de mercancias facturados previamente"),("06","Factura generada por los traslados previos"),
                                                ("07","CFDI por aplicación de anticipo"),("08","Factura generada por pagos en parcialidades"),
                                                ("09","Factura generada por pagos diferidos")].iter().cloned().collect();
    let key =  get_data(&cfdi, "TipoRelacion");
    if let Some(tipo) = tipo_rel.get(key){
        web.eval(&format!("rellenarRelacionadosCabe('{}')", format!("{} - {}", key, tipo)))?;
    }
    Ok(())
}

fn regimen_fiscal(web: &mut web_view::WebView<'_, ()>, cfdi: & Element) -> Result<(), web_view::Error>{
    let tipo_reg: HashMap<&str, &str> = [("601","General de Ley Personas Morales"),("603","Personas Morales con Fines no Lucrativos"),("605","Sueldos y Salarios e Ingresos Asimilados a Salarios"),
    ("606","Arrendamiento"), ("608","Demás ingresos"),("609","Consolidación"),("610","Residentes en el Extranjero sin Establecimiento Permanente en México"),
    ("611","Ingresos por Dividendos (socios y accionistas)"),("612","Personas Físicas con Actividades Empresariales y Profesionales"),("614","Ingresos por intereses"),
    ("615","Régimen de los ingresos por obtención de premios"),("616","Sin obligaciones fiscales"),("620","Sociedades Cooperativas de Producción que optan por diferir sus ingresos"),
    ("621","Incorporación Fiscal"),("622","Actividades Agrícolas, Ganaderas, Silvícolas y Pesqueras"),("623","Opcional para Grupos de Sociedades"),
    ("624","Coordinados"),("628","Hidrocarburos"),("607","Régimen de Enajenación o Adquisición de Bienes"),("629","De los Regímenes Fiscales Preferentes y de las Empresas Multinacionales"),
    ("630","Enajenación de acciones en bolsa de valores")].iter().cloned().collect();
    let key =  get_data(&cfdi, "RegimenFiscal");
    if let Some(tipo) = tipo_reg.get(key){
        web.eval(&format!("rellenar('regimenEmi', 'Regimen Fiscal', '{}')", format!("{} - {}", key, tipo)))?;
    }
    Ok(())
}

fn metodo_pago( cfdi: & Element, tit: &str) -> Result<String, web_view::Error>{
    let key =  get_data(&cfdi, tit);
    let dato: &str;
    if key == "PUE"{
        dato = "PUE - Pago en una sola exhibición";
    }else if key == "PPD"{
        dato = "PPD - Pago en parcialidades o diferido"
    }else{
        dato = key;
    }
    Ok(dato.to_string())
}

fn forma_pago(cfdi: & Element, tit: &str) -> Result<String, web_view::Error>{
    let forma: HashMap<&str, &str> = [("01",	"Efectivo"), ("02",	"Cheque nominativo"), ("03",	"Transferencia electrónica de fondos"),
    ("04",	"Tarjeta de crédito"), ("05",	"Monedero electrónico"), ("06",	"Dinero electrónico"), ("08",	"Vales de despensa"),
    ("12",	"Dación en pago"), ("13",	"Pago por subrogación"), ("14",	"Pago por consignación"), ("15",	"Condonación"),
    ("17",	"Compensación"), ("23",	"Novación"), ("24",	"Confusión"), ("25",	"Remisión de deuda"), ("26",	"Prescripción o caducidad"),
    ("27",	"A satisfacción del acreedor"), ("28",	"Tarjeta de débito"), ("29",	"Tarjeta de servicios"), ("30",	"Aplicación de anticipos"),
    ("31",	"Intermediario pagos"), ("99",	"Por definir")].iter().cloned().collect();
    let key =  get_data(&cfdi, tit);
    if let Some(tipo) = forma.get(key){
        return Ok(format!("{} - {}", key, tipo.to_string()));
    }
    Ok(key.to_string())
}

fn uso_cfdi(web: &mut web_view::WebView<'_, ()>, cfdi: & Element) -> Result<(), web_view::Error>{
    let tipo_rel: HashMap<&str, &str> = [("G01",	"Adquisición de mercancias"), ("G02",	"Devoluciones, descuentos o bonificaciones"),
    ("G03",	"Gastos en general"), ("I01",	"Construcciones"),("I02",	"Mobilario y equipo de oficina por inversiones"),
    ("I03",	"Equipo de transporte"), ("I04",	"Equipo de computo y accesorios"), ("I05",	"Dados, troqueles, moldes, matrices y herramental"),
    ("I06",	"Comunicaciones telefónicas"), ("I07",	"Comunicaciones satelitales"), ("I08",	"Otra maquinaria y equipo"),
    ("D01",	"Honorarios médicos, dentales y gastos hospitalarios."), ("D02",	"Gastos médicos por incapacidad o discapacidad"), ("D03",	"Gastos funerales."),
    ("D04",	"Donativos."), ("D05",	"Intereses reales efectivamente pagados por créditos hipotecarios (casa habitación)."), ("D06",	"Aportaciones voluntarias al SAR."),
    ("D07",	"Primas por seguros de gastos médicos."), ("D08",	"Gastos de transportación escolar obligatoria."), 
    ("D09",	"Depósitos en cuentas para el ahorro, primas que tengan como base planes de pensiones."), ("D10",	"Pagos por servicios educativos (colegiaturas)"),
    ("P01",	"Por definir")].iter().cloned().collect();
    let key =  get_data(&cfdi, "UsoCFDI");
    if let Some(tipo) = tipo_rel.get(key){
        web.eval(&format!("rellenar('usoCfdi', 'Uso CFDI', '{}')", format!("{} - {}", key, tipo)))?;
    }
    Ok(())
}

//hacerlo asincrono o en otro thread
fn validar_cfdi_sat(web: &mut web_view::WebView<'_, ()>, rfc_emit: &str, rfc_recib: &str, total: &str, uuid: &str) -> Result<bool, web_view::Error>{
    let cdata = format!("<![CDATA[?re={}&rr={}&tt={}&id={}]]>", rfc_emit, rfc_recib, total, uuid);
    let cuerpo = format!("<soapenv:Envelope xmlns:soapenv=\"http://schemas.xmlsoap.org/soap/envelope/\" xmlns:tem=\"http://tempuri.org/\"><soapenv:Header/><soapenv:Body><tem:Consulta><!--Optional:--><tem:expresionImpresa>{}</tem:expresionImpresa></tem:Consulta></soapenv:Body></soapenv:Envelope>", cdata);
    // sync post request 
    let resp = ureq::post("https://consultaqr.facturaelectronica.sat.gob.mx/ConsultaCFDIService.svc?wsdl")
    .set("Content-type", "text/xml;charset=\"utf-8\"")
    .set("Content-type", "text/xml;charset=\"utf-8\"")
    .set("Accept", "value: V")
    .set("SOAPAction", "http://tempuri.org/IConsultaCFDIService/Consulta")
    .timeout_connect(5_000) //al hacerlo asincrono puedes dejarlo mas tiempo
    .send_string(&cuerpo);
    //println!("{:?}", resp);
    if resp.ok() {
        if let Ok(res) = resp.into_string(){
            let esta: Vec<&str> = res.split("<a:Estado>").collect();
            if esta.len() == 2{
                let dos: Vec<&str> = esta[1].split("</a:Estado>").collect();
                //println!("{}",dos[0]);
                web.eval(&format!("esValido('{}')", dos[0]))?;
                return Ok(true);
            }else{
                //println!("validar_cfdi_sat: res: {}", res);
                web.eval(&format!("esValido('pendienteOk')"))?;
            }
        }else{
            //println!("validar_cfdi_sat: No se puede convertir a string la respuesta");
            web.eval(&format!("esValido('pendienteOk')"))?;
        }
    } else {
        //println!("validar_cfdi_sat: respuesta distinta a 2xx");
        web.eval(&format!("esValido('pendienteNo2xx')"))?;
    }
    Ok(false)
}