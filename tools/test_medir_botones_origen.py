"""Tests for medir_botones.py: which labels it accepts and where their text came from."""

from medir_botones import pulsar_botones
from medir_inventario import inventariar
from medir_proyectos import abrir
from pvsyst_falso import PVsystFalso, botones_de, formulario, objetivo_de

# pytest finds the autouse fixture through this import; the tests never call it.
from pvsyst_falso import sin_esperas  # noqa: F401


def test_pulsar_botones_presses_only_normalized_white_listed_labels() -> None:
    permitidos = [
        "Resumen del sistema",
        "Gráfico de pérdidas",
        "&Gráfico",
        "Gráfico...",
        "TABLA",
        "Tabla mensual",
        "Mostrar  gráfico",
        "Mostrar histograma",
        "Mostrar optimización",
        "Curva I/V",
        "Diagrama unifilar",
    ]
    no_permitidos = [
        "OK",
        "Aceptar",
        "Sí",
        "Reemplazar",
        "Simular",
        "Ejecutar",
        "Calcular",
        "Actualizar",
        "Despejar",
        "Crear",
        "Elaborar",
        "Conversión",
        "Abortar",
        "Aplicar",
        "Editar",
        "Nuevo",
        "Añadir",
        "Guardar",
        "Eliminar",
        "Pegar",
        "Copiar",
        "Cancelar",
        "Cerrar",
        "Consultar",
        "Ver detalles",
        "Abrir ficha",
        "Detalle",
        "Tabla de modelos",
        "Gráfico de pérdidas detallado",
    ]
    f = formulario([])
    f["paginas"][0]["botones"] = [
        *botones_de(*permitidos, *no_permitidos),
        {"texto": "No habilitado", "habilitado": False},
        {"texto": "", "habilitado": True},
        {"texto": None, "habilitado": True},
    ]
    c = PVsystFalso({"TX": f}, {"a": "TX"}, {t: {"cierra": None} for t in permitidos})
    assert abrir(c, objetivo_de("TX")) is None

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert c.clics == permitidos
    assert [e["boton"] for e in abren if e.get("pulsado") is not False] == permitidos
    no_pulsados = [e for e in abren if e.get("pulsado") is False]
    assert [e["boton"] for e in no_pulsados] == [
        *no_permitidos,
        "No habilitado",
        "",
        "",
    ]
    assert [e["motivo"] for e in no_pulsados] == [
        *["no está en la lista blanca"] * len(no_permitidos),
        "deshabilitado",
        "vacío",
        "vacío",
    ]


def test_pulsar_botones_never_presses_a_label_the_white_list_does_not_equal() -> None:
    """Each of these escaped the old prefix list: one skips a warning, others leave PVsyst."""
    ajenos = [
        "Ver el TNCO de todos modos",
        "Abrir",
        "Abrir sitio",
        "Abrir en explorer",
        "Mostrar mapa",
        "Verificar",
        "Versión",
        "Vertical",
        "Tabla de modelos",
        "Abrir carpeta",
        "Gráfico de pérdidas detallado",
    ]
    f = formulario([])
    f["paginas"][0]["botones"] = botones_de(*ajenos)
    c = PVsystFalso({"TX": f}, {"a": "TX"})
    assert abrir(c, objetivo_de("TX")) is None

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert c.clics == []
    assert abren == [
        {"boton": t, "pulsado": False, "motivo": "no está en la lista blanca"}
        for t in ajenos
    ]


def test_pulsar_botones_does_not_press_a_label_that_is_not_its_own_caption() -> None:
    f = formulario([])
    f["paginas"][0]["botones"] = [
        *botones_de("Tabla", origen="ocr"),
        *botones_de("Gráfico", origen=None),
        *botones_de("Tabla mensual"),
    ]
    c = PVsystFalso(
        {"TX": f},
        {"a": "TX"},
        {t: {"cierra": None} for t in ("Tabla", "Gráfico", "Tabla mensual")},
    )
    assert abrir(c, objetivo_de("TX")) is None

    abren = pulsar_botones(c, objetivo_de("TX"), inventariar(c, "TX"))

    assert c.clics == ["Tabla mensual"]
    assert abren == [
        {"boton": "Tabla", "pulsado": False, "motivo": "texto de OCR"},
        {"boton": "Gráfico", "pulsado": False, "motivo": "sin origen"},
        {"pestana": None, "boton": "Tabla mensual", "abre": [], "error": None},
    ]


def test_pulsar_botones_records_the_origin_of_the_label_it_presses() -> None:
    f = formulario([])
    f["paginas"][0]["botones"] = botones_de("Gráfico")
    c = PVsystFalso({"TX": f}, {"a": "TX"}, {"Gráfico": {"cierra": None}})
    assert abrir(c, objetivo_de("TX")) is None

    medida = inventariar(c, "TX")
    pulsar_botones(c, objetivo_de("TX"), medida)

    assert c.clics == ["Gráfico"]
    assert medida["pestanas"][0]["botones"] == [
        {"texto": "Gráfico", "habilitado": True, "origen": "window_text"}
    ]
