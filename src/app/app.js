'use strict'

window.external.invoke("inicio");

var htmlHide = {
  app: true
}
ocultarElemento("appContainer");

function mostrarApp(isIt){
  if (htmlHide.app && isIt){
    document.getElementById("appContainer").style.display="flex";
  }else if(!htmlHide.app && !isIt){
    document.getElementById("appContainer").style.display="none";
  }
  htmlHide.app= !htmlHide.app;
}

function showConcep(isIt){
  if (isIt){
    document.getElementById("conceptos").innerHTML=`
    <div class="concepCont" id="concepto_-1">
      <div class="concepBase cabrows borde">
        <div class="cantidad"><b>Cantidad</b></div>
        <div class="precioUni"><b>Precio Unitario</b></div>
        <div class="descrip"><b>Descripción</b></div>
        <div class="descConcep"><b>Descuento</b></div>
        <div class="importeConcep"><b>Importe</b></div>
      </div>
    </div>
    `;
  }else{
    document.getElementById("conceptos").innerHTML="";
  }
}

function addConcep(idx, cantidad, claveUnidad, unidad, claveProdServ, numIde, valorUni, descrip, importe, descuento){
  //quitar lo que no se tiene en los adicionales
  if(unidad != ""){
    var unidad = `<div class="unidad datoContCenter"><div class="datoTit"><b>Unidad:</b></div><div class="datoCab">${unidad}</div></div>`;
  }
  if(claveUnidad != ""){
    var claveUnidad = `<div class="claveUnidad datoContCenter"><div class="datoTit"><b>Clave de Unidad:</b></div><div class="datoCab">${claveUnidad}</div></div>`;
  }
  if(claveProdServ != ""){
    var claveProdServ = `<div class="claveProdServ datoContCenter"><div class="datoTit"><b>Clave Producto/Servicio:</b></div><div class="datoCab"> ${claveProdServ}</div></div>`;
  }
  if(numIde != ""){
    var numIde = `<div class="noIdent datoContCenter"><div class="datoTit"><b>Num. Identificación:</b></div><div class="datoCab"> ${numIde}</div></div>`;
  }
  var concep = `
  <div class="concepCont" id="concepto_${idx}">
      <div class="concepBase cabrows borde pointer" id="concepBase_${idx}">
        <div class="cantidad">${cantidad}</div>
        <div class="precioUni">${valorUni}</div>
        <div class="descrip">${descrip}</div>
        <div class="descConcep">${descuento}</div>
        <div class="importeConcep">${importe}</div>
      </div>
      <div class="concepDet" id="concepDet_${idx}">
        <div class ="datosAdCol borde" id="datosAd_${idx}">
          ${unidad}
          ${claveUnidad}
          ${claveProdServ}
          ${numIde}
        </div>
      </div>
  </div>
  `;
  document.getElementById(`concepto_${idx-1}`).insertAdjacentHTML("afterend", concep);
  document.getElementById(`concepDet_${idx}`).style.display="none";
  //evento al click
  document.getElementById(`concepBase_${idx}`).addEventListener("click", ()=>{
    var disp = document.getElementById(`concepDet_${idx}`).style.display;
    if(disp == "none"){
      document.getElementById(`concepDet_${idx}`).style.display="flex";
      document.getElementById(`concepBase_${idx}`).classList.add("pointerClick");
      document.getElementById(`concepBase_${idx}`).classList.remove("pointer");
    }else{
      document.getElementById(`concepBase_${idx}`).classList.remove("pointerClick");
      document.getElementById(`concepBase_${idx}`).classList.add("pointer");
      document.getElementById(`concepDet_${idx}`).style.display="none";
    }
  })
}

function trasRetConcepCabe(idx){
  document.getElementById(`datosAd_${idx}`).insertAdjacentHTML("afterend", `<div class="borde trasRetConcepCont" id="trasRetConcepCont_${idx}">
                                                                              <div id="trasRetConcep_${idx}_-1" class="retConcepCont bordeDownDash datosAdCol">
                                                                              <div class="datoContTrasRetCabe"><b>Tipo</b></div>
                                                                              <div class="datoContTrasRetCabe"><b>Impuesto</b></div>
                                                                              <div class="datoContTrasRetCabe"><b>Base a Aplicar Impuesto</b></div>
                                                                              <div class="datoContTrasRetCabe"><b>Tipo de Factor</b></div>
                                                                              <div class="datoContTrasRetCabe"><b>Tasa o Cuota</b></div>
                                                                              <div class="datoContTrasRetCabe"><b>Importe de Impuesto</b></div>
                                                                              </div>
                                                                            </div>`);
}

function addTrasRetConcep(idx, id, tipo, importe, tasa, tipoFactor, impuesto, base){
  var name = "";
  if(tipo == "Traslados"){
    name = "Traslado:";
  }else if(tipo == "Retenciones"){
    name = "Retención:";
    importe = "- "+importe;
  }
  if (tipoFactor == "Tasa"){
    tasa = (Number(tasa)*100).toFixed(4)+"%";
  }
  document.getElementById(`trasRetConcep_${idx}_${id-1}`).insertAdjacentHTML("afterend", `<div id="trasRetConcep_${idx}_${id}" class="retConcepCont datosAdCol">
                                                                              <div class="datoContTrasRet"><b>${name}</b></div>
                                                                              <div class="datoContTrasRet">${impuesto}</div>
                                                                              <div class="datoContTrasRet">${base}</div>
                                                                              <div class="datoContTrasRet">${tipoFactor}</div>
                                                                              <div class="datoContTrasRet">${tasa}</div>
                                                                              <div class="datoContTrasRet">${importe}</div>
                                                                            </div>`);
}
//agrega la cabecera a la tabla de los impuestos generales
function impuestosCabe(){
  document.getElementById("impuestos").innerHTML = `<div class="datoCont bordeUpDash bordeDownDash impCont datosAdCol" id="impCont_-1">
                                                      <div class="datoContImp"><b>Tipo</b></div>
                                                      <div class="datoContImp"><b>Impuesto</b></div>
                                                      <div class="datoContImp"><b>Factor</b></div>
                                                      <div class="datoContImp"><b>Tasa o Cuota</b></div>
                                                      <div class="datoContImp datoD"></div>
                                                  </div>`
}
//agrega los traslados y retenciones dentro del nodo impuestos general
function addTrasRetImp(idx, tipo, impuesto, factor, tasa, importe){
  if(tipo == "Retencion"){
    importe = "- "+importe;
  }
  if (factor == "Tasa"){
    tasa = (Number(tasa)*100).toFixed(4)+"%";
  }
  document.getElementById(`impCont_${idx-1}`).insertAdjacentHTML("afterend", `<div class="datoCont impCont datosAdCol" id="impCont_${idx}">
                                                                                <div class="datoContImp">${tipo}</div>
                                                                                <div class="datoContImp">${impuesto}</div>
                                                                                <div class="datoContImp">${factor}</div>
                                                                                <div class="datoContImp">${tasa}</div>
                                                                                <div class="datoContImp datoD">${importe}</div>
                                                                              </div>`);
}
function rellenar(id, titulo, data){
  if(data != ""){
    document.getElementById(id).innerHTML=`<div class="datoCont"><div class="datoTit"><b>${titulo}: </b></div><div class="datoD">${data}</div></div>`;
  }else{
    ocultarElemento(id);
  }
}

function rellenarCabe(id, titulo, data){
  if(data != ""){
    document.getElementById(id).innerHTML=`<div class="datoCont borde">
                                            <div class="datoTit">
                                              <b>${titulo}: </b>
                                            </div>
                                            <div class="datoCab">${data}</div>
                                          </div>`;
  }else{
    ocultarElemento(id);
  }
}
    
function ponerQr(data){
  if (data != ""){
    var qrCode = new QRCode("qr", {
      text: data,
      width: 512,
      height: 512,
      colorDark: "#232323",
      colorLigth: "#f9f9f9",
      correctLevel: QRCode.CorrectLevel.H
    });
  }else{
    ocultarElemento(id);
  }
}

function rellenar_cortado(id, titulo, data){
  if(data != ""){
    document.getElementById(id).innerHTML=`<div class=""><div class="datoTit"><b>${titulo}: </b></div><div class="cortarStr">${data}</div></div>`;
  }else{
    ocultarElemento(id);
  }
}

function rellenarRelacionadosCabe(tipo){
  //crear dentro de un div de relacionados un bloque para poner los relacionados de este nodo
  document.getElementById("relacionadosAll").innerHTML = `<div id="relacionadosCab" class="borde">
                                                            <div class="cabrows bordeDownDash" id="relacionadosTit">
                                                              <div class="datoTit" ><b>Documentos Relacionados:</b></div>
                                                              <div class="datoCont" id="tipoRelacion">
                                                                <div class="datoTit"><b>Tipo de Relación:</b></div>
                                                                <div class="datoCab"> ${tipo}</div>
                                                              </div>
                                                            </div>
                                                            <div class="" id="relacionadoCab"></div>
                                                          </div>`
}

function addRelacionado(id, uuid){
  if (uuid != "") {
    var info = `<div id="relacion_${id}" class="datoCont"><div class="datoTit"><b>Comprobante Relacionado UUID:</b></div><div class="datoCab">${uuid}</div>`
    if (id == 0){
      document.getElementById("relacionadoCab").innerHTML=info;
    }else{
      document.getElementById(`relacion_${idx-1}`).insertAdjacentHTML("afterend", info);
    }
  }
}

function esValido(val){
  if(val == "Vigente"){
    document.getElementById("validar").innerHTML=`<b class="success padText">Comprobante Valido y Vigente</b>`;
  }else if(val == "pendienteOk"){
    document.getElementById("validar").innerHTML=`<b class="warning padText">Respuesta Incompleta del Servicio de Validación, Intentelo Mas Tarde</b>`;
  }else if(val == "pendienteNo2xx"){
    document.getElementById("validar").innerHTML=`<b class="warning padText">Sin Respuesta del Servicio de Validación, Intentelo Mas Tarde</b>`;
  }else{
    document.getElementById("validar").innerHTML=`<b class="danger padText">Respuesta de Validación: ${val}</b>`;
  }
}

function pagosCont(){
  document.getElementById("pagos").innerHTML=`<div class="borde" id="pagosCont">
                                                <div class="" id=""><b>Complemento de Pago</b></div>
                                                <div class="cabrows bordeDownDash bordeUpDash" id="pagoCabe_-1">
                                                  <div class="" id="fechaP"><b>Fecha de Pago</b></div>
                                                  <div class="" id="formaPagoP"><b>Forma de Pago</b></div>
                                                  <div class="" id="monedaP"><b>Moneda de Pago</b></div>
                                                  <div class="" id="montoP"><b>Monto de Pago</b></div>
                                                </div>
                                              </div>`
}

function pagoCabe(id, fechaP, formaPP, monedaP, montoP, otros){
  var pago = `<div class="pagoCont" id="pagoCabe_${id}">
                <div class="cabrows borde pointer" id="pagoBase_${id}">
                  <div class="" id="fechaP">${fechaP}</div>
                  <div class="" id="formaPagoP">${formaPP}</div>
                  <div class="" id="monedaP">${monedaP}</div>
                  <div class="" id="montoP">${montoP}</div>
                </div>
                <div class="" id="pagoDet_${id}">
                  <div class="pagoAd cabrows">demas datos</div>
                  <div class="docRelCont" id="docRelCont_${id}"></div>
                </div>
              </div>`;
  document.getElementById(`pagoCabe_${id-1}`).insertAdjacentHTML("afterend", pago);
  document.getElementById(`pagoDet_${id}`).style.display="none";
  //evento al click
  document.getElementById(`pagoBase_${id}`).addEventListener("click", ()=>{
    var disp = document.getElementById(`pagoDet_${id}`).style.display;
    if(disp == "none"){
      document.getElementById(`pagoDet_${id}`).style.display="flex";
      document.getElementById(`pagoBase_${id}`).classList.add("pointerClick");
      document.getElementById(`pagoBase_${id}`).classList.remove("pointer");
    }else{
      document.getElementById(`pagoBase_${id}`).classList.remove("pointerClick");
      document.getElementById(`pagoBase_${id}`).classList.add("pointer");
      document.getElementById(`pagoDet_${id}`).style.display="none";
    }
  })    
}

function debug(data){
  window.external.invoke(data);
}
function ocultarElemento(id){
  document.getElementById(id).style.display="none";
}