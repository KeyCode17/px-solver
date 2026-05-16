/*
 * appId: eT15wiaE (companion file, not the PX bundle)
 * captured: 2026-05-16
 * vendor: HUMAN Security / PedidosYa custom shim
 * method: curl
 * source_url: https://live.pystatic.com/pxassets/personalization2.js
 * sanitized_with: scripts/sanitize_capture.sh
 * size_raw_bytes: 4483
 * notes: tiny PedidosYa-side customization layer for the PX block/captcha
 *        UI. Reads window._pxUuid, sets window._pxSelectedLocale = "es-419"
 *        and overrides window._pxTranslation. Confirms that operators can
 *        customize the user-facing block page even without touching the
 *        PX bundle itself.
 */
let fontLink = document.createElement("link");
fontLink.rel = "stylesheet";
fontLink.href = "https://fonts.googleapis.com/css?family=Muli:600,800%7CLato:400,700&display=swap";
document.head.appendChild(fontLink);
const refId = window._pxUuid;
window._pxSelectedLocale = "es-419";
window._pxTranslation = {
    "es-419": [{
        selector: "title",
        text: "Acceso ha sido denegado | PedidosYa",
    }, {
        selector: ".page-title h1",
        text: "Por favor confirma que eres un humano",
    }, {
        selector: ".content-wrapper p:nth-of-type(1)",
        text: "El acceso a esta página ha sido denegado porque detectamos tráfico inusual procedente de tu red de computadoras.",
    }, {
        selector: ".content-wrapper p:nth-of-type(2)",
        text: "Esto puede suceder como resultado de alguna de las siguientes posibles causas:",
    }, {
        selector: ".content-wrapper li:nth-of-type(1)",
        text: "El envío de búsquedas en múltiples páginas muy rápidamente.",
    }, {
        selector: ".content-wrapper li:nth-of-type(2)",
        text: "La misma red desde donde te conectás a Internet está siendo usada por muchos usuarios.",
    }, {
        selector: ".content-wrapper p:nth-of-type(3)",
        text: "Por favor asegúrate que Javascript y cookies están habilitadas en tu navegador y que no se está bloqueando su carga.",
    }, {
        selector: ".content-wrapper p:nth-of-type(4)",
        text: "Número de referencia: #" + refId,
    }, {
        selector: "#px-form-title",
        text: "Reportar un problema",
    }, {
        selector: "#px-form-subtitle",
        text: "¿Estás teniendo problemas con esta página? Por favor avísanos:",
    }, {
        selector: "#px-form-item-ref-id",
        text: "Para contactarnos por ayuda usa el número de referencia:",
    }, {
        selector: "#px-form-item-options-title",
        text: "Puedes enviarnos tu opinión:",
    }, {
        selector: "#px-form-item-option-1",
        text: "No veo donde confirmar",
    }, {
        selector: "#px-form-item-option-2",
        text: "Resolví varios captchas, sigo sin poder acceder",
    }, {
        selector: "#px-form-item-option-3",
        text: "Otro (por favor elabore en comentarios)",
    }, {
        selector: "#px-form-textarea-title",
        text: "Información adicional:",
    }, {
        selector: "#px-form-cancel",
        text: "Cancelar",
    }, {
        selector: "#px-form-submit",
        text: "Enviar",
    }, {
        selector: "#px-form-thank-you-text",
        text: "¡Gracias por tus comentarios!"
    }, {
        selector: "#px-block-toggle-button",
        text: "Reportar un problema",
    }, {
        selector: "#px-form textarea",
        text: "¿Experimentando otros problemas?",
        attribute: "placeholder",
    }, ],
};
window._PXeT15wiaE = {
    "challenge": {
      "view": {
        "textColor": "#0d0c0c",
        "textFont": "Lato, sans-serif",
        "borderColor": "#000000",
        "borderWidth": 2,
        "fillColor": "#000000"
      },
      "translation": {
        "default": {
          "btn": "PULSAR Y MANTENER PULSADO"
        }
      },
      "context": {
        "messageText": "El acceso a esta página ha sido denegado porque detectamos tráfico inusual procedente de tu red de computadoras.",
        "messageFontWeight": "400",
        "messageColor": "#626363",
        "headerText": "Por favor confirma que eres un humano",
        "headerFontFamily": "Lato, sans-seriff",
        "headerFontWeight": "500"
      }
    }
};
window._PXmhJnWOUv = {
    "challenge": {
      "view": {
        "textColor": "#0d0c0c",
        "textFont": "Lato, sans-serif",
        "borderColor": "#000000",
        "borderWidth": 2,
        "fillColor": "#000000"
      },
      "translation": {
        "default": {
          "btn": "PULSAR Y MANTENER PULSADO"
        }
      },
      "context": {
        "messageText": "El acceso a esta página ha sido denegado porque detectamos tráfico inusual procedente de tu red de computadoras.",
        "messageFontWeight": "400",
        "messageColor": "#626363",
        "headerText": "Por favor confirma que eres un humano",
        "headerFontFamily": "Lato, sans-seriff",
        "headerFontWeight": "500"
      }
    }
};

let elem;
for (let l of window._pxTranslation[window._pxSelectedLocale]) {
    elem = document.querySelector(l.selector);
    if (elem) {
        elem.innerText = l.text;
    }
}



