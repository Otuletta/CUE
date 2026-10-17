# CUE
## Documento maestro de producto, arquitectura e implementación

**Nombre del proyecto:** CUE  
**Naturaleza:** motor de billar y experiencia de juego web, modular y de código abierto  
**Documento:** especificación funcional y técnica / *Product & Engineering Blueprint*  
**Versión del documento:** 1.0.0  
**Fecha de referencia:** 17 de octubre de 2026  
**Estado:** propuesta de arquitectura aprobada conceptualmente; implementación pendiente  
**Idioma del documento:** español  
**Lenguaje principal del producto:** Rust  
**Destino inicial:** integración dentro de un portfolio personal, sin aplicación web independiente  

> **Regla de interpretación.** Este documento describe el producto que se quiere construir, no funcionalidades que ya existan. Las API, rutas, nombres de paquetes, presupuestos y fragmentos de código son **contratos propuestos** hasta que se implementen y validen. Las dependencias se deben fijar en un `Cargo.lock` y contrastar antes de iniciar cada fase. No se prometen resultados de precisión, compatibilidad ni rendimiento sin pruebas.

---

## Índice

1. [Resumen ejecutivo](#1-resumen-ejecutivo)
2. [Identidad, misión y propuesta de valor](#2-identidad-misión-y-propuesta-de-valor)
3. [Decisiones de producto y límites del alcance](#3-decisiones-de-producto-y-límites-del-alcance)
4. [Público objetivo y escenarios de uso](#4-público-objetivo-y-escenarios-de-uso)
5. [Objetivos, principios y métricas](#5-objetivos-principios-y-métricas)
6. [Alcance funcional por versiones](#6-alcance-funcional-por-versiones)
7. [Experiencia de usuario y diseño visual](#7-experiencia-de-usuario-y-diseño-visual)
8. [Modos de juego](#8-modos-de-juego)
9. [Reglamento de bola ocho](#9-reglamento-de-bola-ocho)
10. [Controles e interacción](#10-controles-e-interacción)
11. [Física y simulación](#11-física-y-simulación)
12. [Gráficos, escena y recursos](#12-gráficos-escena-y-recursos)
13. [Audio y respuesta sensorial](#13-audio-y-respuesta-sensorial)
14. [Stack tecnológico completo](#14-stack-tecnológico-completo)
15. [Arquitectura de software](#15-arquitectura-de-software)
16. [Modelo de datos, estados y eventos](#16-modelo-de-datos-estados-y-eventos)
17. [API pública y contrato de integración](#17-api-pública-y-contrato-de-integración)
18. [Integración en el portfolio](#18-integración-en-el-portfolio)
19. [Estructura del repositorio](#19-estructura-del-repositorio)
20. [Preparación, desarrollo y compilación](#20-preparación-desarrollo-y-compilación)
21. [Rendimiento y compatibilidad](#21-rendimiento-y-compatibilidad)
22. [Accesibilidad y experiencia responsable](#22-accesibilidad-y-experiencia-responsable)
23. [Privacidad, seguridad y operación](#23-privacidad-seguridad-y-operación)
24. [Pruebas y aseguramiento de calidad](#24-pruebas-y-aseguramiento-de-calidad)
25. [Documentación, licencia y comunidad open source](#25-documentación-licencia-y-comunidad-open-source)
26. [CI/CD, distribución y versiones](#26-cicd-distribución-y-versiones)
27. [Roadmap de ejecución](#27-roadmap-de-ejecución)
28. [Backlog funcional detallado](#28-backlog-funcional-detallado)
29. [Riesgos y medidas de mitigación](#29-riesgos-y-medidas-de-mitigación)
30. [Criterios de aceptación y definición de terminado](#30-criterios-de-aceptación-y-definición-de-terminado)
31. [Decisiones abiertas y registro de arquitectura](#31-decisiones-abiertas-y-registro-de-arquitectura)
32. [Glosario](#32-glosario)
33. [Fuentes técnicas y normativas](#33-fuentes-técnicas-y-normativas)

---

## 1. Resumen ejecutivo

**CUE es un motor de billar interactivo, open source, escrito principalmente en Rust y compilable a WebAssembly, acompañado de una experiencia jugable que se incrusta directamente en un sitio web.** Su primera implementación será una sección del portfolio del autor. No exige crear una cuenta, iniciar sesión, pagar, conectarse a un servidor propio ni abandonar el portfolio para jugar.

El producto tiene **dos caras inseparables**:

- **Experiencia para visitantes:** un juego de billar de calidad, accesible desde el navegador, con controles naturales, mesa tridimensional, físicas convincentes, modo libre y partida local de bola ocho.
- **Herramienta para desarrolladores:** código reutilizable, módulos con responsabilidades claras y una API mínima que permita incrustar CUE en webs de terceros sin copiar todo el proyecto ni adoptar el framework del portfolio.

La prioridad no es desarrollar una red social, una plataforma competitiva ni una réplica de *8 Ball Pool*. La prioridad es **una mesa extraordinariamente bien resuelta**, desde la sensación de golpear la bola hasta el movimiento, el sonido, el rendimiento y la calidad del empaquetado.

### 1.1. Resultado esperado

Un visitante entra en la sección `/cue` del portfolio, ve una presentación visual limpia, pulsa «Jugar» y se abre la mesa sin autenticación ni navegación externa. Puede disparar, reiniciar, practicar o jugar con otra persona en el mismo dispositivo. El juego ocupa el espacio disponible, se adapta a la pantalla y puede utilizarse en modo pantalla completa cuando el navegador lo permita.

Otro desarrollador puede instalar o incorporar el paquete público, montar una instancia de la mesa en un contenedor de su página, recibir eventos de juego, controlar su ciclo de vida y cambiar determinados parámetros de presentación.

### 1.2. Propuesta técnica en una frase

**Rust + Bevy 0.20 + `bevy_rapier3d` 0.37 + WebAssembly + `wasm-bindgen` + capa mínima de JavaScript, con renderizado web orientado inicialmente a WebGL2.** Las versiones citadas son las comprobadas en la fecha del documento; se bloquearán en el repositorio y se revisarán antes del arranque.

## 2. Identidad, misión y propuesta de valor

### 2.1. Qué es CUE

CUE es una biblioteca/motor de billar con una demostración jugable de referencia. Gestiona la simulación, las reglas, los estados de juego, la representación visual, los controles y la integración web mediante módulos separados.

### 2.2. Qué no es CUE

CUE **no** es, en su alcance inicial:

- Un SaaS, producto con suscripciones o sistema de cuentas.
- Una página comercial independiente del portfolio.
- Una plataforma de partidas en línea o emparejamiento.
- Una copia visual o funcional literal de un juego de terceros.
- Un motor genérico de cualquier juego deportivo.
- Un simulador científico certificado o un sustituto de mediciones físicas de laboratorio.
- Una aplicación que necesite una API de servidor para ejecutar cada tiro.

### 2.3. Misión

Hacer que el billar digital resulte natural, atractivo y fácil de integrar, demostrando al mismo tiempo capacidades reales de programación de sistemas, simulación, arquitectura modular y software web mediante Rust y WebAssembly.

### 2.4. Valores de diseño

**Precisión perceptible; interacción inmediata; sofisticación sin exceso; modularidad real; rendimiento responsable; documentación transparente; accesibilidad progresiva; independencia del host.**

### 2.5. Diferenciación

1. **Juego primero:** no será una demostración técnica que solo luce bien en vídeo; se tiene que poder disfrutar.
2. **Librería primero:** no dependerá internamente del router, estilos ni estado de la web que la incruste.
3. **Rust real:** simulación, reglas y lógica principal serán Rust, no TypeScript disfrazado detrás de un componente.
4. **Visual propio:** no reproducir interfaces, logotipos, recursos ni mecánicas exclusivas de marcas comerciales.
5. **Configurabilidad sensata:** temas, controles, cámara, sonidos y parámetros públicos sin exponer directamente todo el motor.
6. **Privacidad por defecto:** la experiencia base no requiere datos personales ni un servicio remoto.

## 3. Decisiones de producto y límites del alcance

### 3.1. Decisiones confirmadas para el diseño

| Área | Decisión |
|---|---|
| Producto | Motor de billar interactivo y experiencia jugable open source |
| Implantación inicial | Integrado en una ruta/sección del portfolio |
| Login y cuentas | Excluidos |
| Backend | No requerido para el producto base |
| Base de datos remota | No requerida |
| Lenguaje principal | Rust |
| Motor de juego/render | Bevy |
| Física inicial | Rapier 3D mediante `bevy_rapier3d` |
| Plataforma objetivo | Navegador, usando WebAssembly |
| Código de integración web | JavaScript mínimo, sin obligar al host a usar React o TypeScript |
| Experiencia inicial | Mesa jugable + Free Play + 8-Ball local |
| Desarrollo | Repositorio independiente del portfolio |
| Distribución prevista | GitHub; paquete web/npm cuando el empaquetado esté validado |

### 3.2. Propuestas pendientes de prueba

| Propuesta | Estado | Condición de adopción |
|---|---|---|
| Física exclusivamente 3D | Preferida | Aprobar prueba de spin, bandas y rendimiento móvil |
| Motor propio de física de billar | Alternativa futura | Se activa si el resultado con Rapier no supera las pruebas de aceptación |
| WebGL2 como ruta de compatibilidad | Preferida | Superar matriz real de navegadores |
| WebGPU | Experimental/futuro | Funcionar de forma estable y aportar un beneficio medible |
| API `createCue(...)` | Contrato sugerido | Prototipo funcional, ciclo de vida correcto y distribución |
| Adaptadores React/Vue/Svelte | Opcionales | Demanda y estabilidad del paquete web base |
| Licencia MIT | Intención | Auditoría de dependencias y autoría de recursos |
| Modo de desafíos | Futuro | Base Free Play y 8-Ball consolidadas |

### 3.3. Funciones expresamente fuera de la primera versión pública

Multijugador online; chat; amigos; cuentas; ranking global; compras; anuncios; skins de pago; editor completo de niveles; inteligencia artificial competitiva sofisticada; realidad virtual; lanzamiento para tiendas móviles; comunidad integrada; físicas de saltos/masés profesionales si no están verificadas; sincronización entre dispositivos.

## 4. Público objetivo y escenarios de uso

| Perfil | Necesidad | Comportamiento esperado |
|---|---|---|
| Visitante casual | Probar algo interesante en el portfolio | Entra, juega en segundos, sale sin compromiso |
| Reclutador/evaluador técnico | Valorar calidad de ingeniería | Juega, visita GitHub, comprende arquitectura y decisiones |
| Desarrollador frontend | Incrustar billar en una web | Usa API pequeña sin conocer Bevy |
| Desarrollador Rust | Estudiar/ampliar el motor | Reutiliza crates y ejecuta pruebas sin navegador |
| Jugadores locales | Competir en el mismo dispositivo | Alternan turnos en 8-Ball |
| Usuario móvil | Interactuar con gestos | Apunta, regula fuerza y dispara sin depender de hover |

### 4.1. Historias de usuario representativas

- Como visitante quiero comenzar sin cuenta, para no invertir tiempo en registros.
- Como jugador quiero que la dirección y potencia del golpe respondan a mis acciones, para sentir control.
- Como jugador quiero distinguir cuándo he cometido una falta y de quién es el turno.
- Como usuario móvil quiero controles cómodos aunque la pantalla sea pequeña.
- Como desarrollador quiero montar y destruir el juego desde mi propia interfaz sin fugas de recursos.
- Como colaborador quiero ejecutar pruebas de físicas y reglas sin descargar ni construir la web del portfolio.

## 5. Objetivos, principios y métricas

### 5.1. Objetivos funcionales

- Mesa tridimensional con seis troneras, bolas numeradas y taco manipulable.
- Juego desde navegador sin permisos ni instalaciones especiales.
- Movimientos, contactos, bandas y entrada en troneras coherentes.
- Free Play de acceso inmediato.
- 8-Ball local de dos jugadores, con reglas documentadas.
- Interfaz discreta, sonido opcional, reinicio, pausa y pantalla completa cuando aplique.
- Controles de ratón, teclado y táctiles diseñados para cada contexto.
- API incrustable y documentación reproducible.

### 5.2. Objetivos de ingeniería

- Separación de lógica y representación.
- Simulación a paso fijo y estados reproducibles dentro de tolerancias definidas.
- Especificación de eventos tipados.
- Pruebas unitarias, de propiedades, integración y navegador.
- Dependencias auditadas y versiones compatibles.
- Empaquetado estático sin backend propio.
- Limpieza correcta de listeners, canvas, audio y memoria al cerrar/desmontar.

### 5.3. Indicadores de éxito propuestos

| Indicador | Cómo se medirá | Criterio objetivo inicial, sujeto a prueba |
|---|---|---|
| Primera interacción | Tiempo desde clic de jugar hasta taco operable | Sin bloqueo innecesario; medir P50/P95 por dispositivo |
| Fluidez desktop | FPS y tiempo de fotograma en equipos de referencia | Aspirar a 60 FPS sostenidos |
| Fluidez móvil | FPS y estabilidad térmica | Aspirar a ≥30 FPS sostenidos en gama media definida |
| Simulación | Casos físicos de referencia | Todos los casos críticos dentro de tolerancias aceptadas |
| Fiabilidad de reglas | Casos automáticos por transición | 100 % de casos normativos implementados para el alcance declarado |
| Integrabilidad | Montar/pausar/destruir/remontar | Sin errores ni listeners/recursos residuales detectables |
| Accesibilidad | Auditoría manual y automatizada | Controles HTML accesibles y alternativa de teclado utilizable |
| Portabilidad | Matriz de navegadores | Sin errores bloqueantes en navegadores soportados |

**Nota:** las métricas son objetivos verificables, no cifras de rendimiento demostradas. Deben definirse máquinas de prueba, escenario fijo y versión exacta antes de aprobarlas.

### 5.4. Principios de priorización

1. Correctitud del motor y claridad de controles.
2. Interacción agradable y visual nítida.
3. Compatibilidad web y consumo razonable de recursos.
4. Integración modular y documentación.
5. Pulido, modos adicionales y efectos decorativos.

## 6. Alcance funcional por versiones

### 6.1. Prototipo físico (*technical spike*)

Una mesa sencilla, bola blanca y al menos dos bolas objetivo, un tiro controlable, colisiones, bandas, detección de inmovilidad y métricas de rendimiento. Gráficos provisionales y sin reglas completas. Su propósito es **demostrar que el stack es viable**.

### 6.2. MVP / primera experiencia usable

Mesa completa con 16 bolas y seis troneras; taco; selección de dirección y fuerza; físicas calibradas inicialmente; tiros repetibles; reinicio; Free Play; HUD mínimo; escritorio y móvil básico; integración con la sección `/cue`; audio configurable; pausa/resumen; modo pantalla completa opcional; manejo de errores de navegador.

### 6.3. Versión pública 1.0

Incluye todo el MVP y, además: 8-Ball local completo según el perfil de reglas declarado; llamada de bola/tronera; faltas automatizables; cambios de turno; controles accesibles; guía de integración; paquete reutilizable; pruebas automatizadas; política de compatibilidad; seguridad mínima; licencia y assets auditados; documentación y ejemplo desacoplado del portfolio.

### 6.4. Evolución 1.x / 2.x (no comprometida)

Desafíos compartibles; presets de mesa y temas; edición de disposiciones; práctica de precisión; replays de golpes; modo espectador; modos alternativos; IA local; mejora de efectos; implementación específica del motor físico si se justifica; builds nativas de escritorio; WebGPU cuando sea suficientemente interoperable.

## 7. Experiencia de usuario y diseño visual

### 7.1. Principio rector

**La mesa es la interfaz.** Los elementos secundarios —puntuación, potencia, ayuda, nombres de jugadores y menú— deben comunicar lo necesario sin competir visualmente con la acción.

### 7.2. Trayecto de usuario

1. Se navega a `/cue` dentro del portfolio.
2. Se ve una presentación liviana del proyecto; no se descarga innecesariamente todo el motor en la home del portfolio.
3. Se solicita «Jugar» y se carga la escena bajo demanda.
4. Se presenta Free Play directamente o un selector discreto de modo (decisión de UX a validar).
5. Aparece una orientación de controles de una sola vez, descartable.
6. El usuario apunta, selecciona fuerza, dispara y recibe respuesta visual/sonora.
7. Puede reiniciar, cambiar modo, pausar, silenciar, pedir ayuda o salir.
8. Al salir, CUE libera recursos y devuelve el control al portfolio.

### 7.3. Composición de pantalla

- Mesa en el centro, con proporciones físicas conservadas y márgenes adecuados.
- Vista cenital/ligeramente oblicua elegida para lectura precisa; la cámara no debe sabotear el apuntado.
- Indicador de fuerza contextual, oculto cuando no se necesita.
- Menú de pausa compacto; nunca un dashboard.
- Información de turno y grupos solo en 8-Ball.
- Trayectorias de ayuda opcionables; su nivel de asistencia deberá declararse.
- Animaciones de transición cortas y desactivables si el usuario prefiere movimiento reducido.
- Resaltados y estados distinguibles sin depender exclusivamente del color.

### 7.4. Dirección artística

Identidad original, minimalista y sofisticada. Materiales con reflejos controlados; texturas de paño discretas; madera/metal u otra terminación coherente; bolas con numeración legible; iluminación suave; sombras físicamente convincentes pero optimizadas. **Sin efectos brillantes por defecto que dificulten la visibilidad de bolas, bandas o taco.**

### 7.5. Sistema de temas propuesto

- `classic`: mesa tradicional sobria.
- `studio`: presentación moderna y refinada para el portfolio.
- `contrast`: prioriza contraste y legibilidad.

Los temas cambian **materiales y presentación**, nunca radios, masa, geometría física o reglas sin una configuración explícita independiente.

### 7.6. Pantallas y estados visuales

`preview`, `loading`, `ready`, `tutorial`, `aiming`, `charging`, `shot_in_progress`, `shot_result`, `paused`, `game_over`, `unsupported`, `error`.

Cada estado debe definir qué botones y controles aparecen, quién tiene el foco del teclado y cuáles son las acciones permitidas.

## 8. Modos de juego

### 8.1. Free Play

**Propósito:** experimentar sin presión ni reglas competitivas.

Comportamientos:

- Disponibilidad inmediata tras la carga.
- Posiciones iniciales estándar de las bolas y reinicio a rack completo.
- Apuntar y tirar sin turnos obligatorios ni penalización por falta.
- Introducir bolas en troneras y mantenerlas retiradas hasta reinicio (política inicial).
- Si la bola blanca entra en tronera, ofrecer reposicionamiento automático o manual en una posición válida.
- Reiniciar solo el tiro **no equivale** a deshacer físicamente el mundo: si se implementa «deshacer», debe restaurar un snapshot íntegro previo al tiro.
- Modo de ayudas opcional, sin resultados globales.

### 8.2. 8-Ball Local

**Propósito:** partida para dos personas en un único dispositivo, por turnos.

Comportamientos:

- Jugadores A y B; se permite un alias local temporal no identificativo.
- Configuración de primer saque y turno inicial.
- Rack legal y bola blanca disponible para saque.
- Estado de mesa abierta, asignación de grupos, bolas restantes y objetivo permitido.
- Llamada de bola y tronera (excepto saque), o declaración de *safety*, según reglas oficiales elegidas.
- Detección de bola objetivo inicial, banda posterior, bolas embolsadas, scratch y situaciones reglamentarias automatizables.
- Cambio de turno y bola en mano donde proceda.
- Determinación de victoria/derrota; resumen de partida y reinicio.
- Si una decisión exige interpretación arbitral difícil de automatizar (por ejemplo, conducta antideportiva), documentar política de adjudicación manual o exclusión explícita. No fingir arbitraje perfecto.

### 8.3. Challenges — fase posterior

Configuración reproducible de bolas, objetivo particular, límite de tiros, puntuación local y posibilidad de compartir un desafío mediante URL codificada y validada. No requiere clasificaciones ni cuentas. Cualquier enlace deberá limitar tamaño y parámetros para evitar estados maliciosos o inválidos.

## 9. Reglamento de bola ocho

### 9.1. Referencia normativa

Para el modo `eight-ball-standard`, la referencia es **World Pool-Billiards Association (WPA), *Rules of Play*, edición efectiva 2025-09-15, sección 4 (8-Ball) y normas generales aplicables**. Documento oficial: <https://www.wpapool.com/wp-content/uploads/2026/01/2026.01.02-WPA-Rules.pdf>.

CUE no debe anunciar cumplimiento íntegro de las reglas WPA hasta haber implementado y probado cada caso aplicable. Si se publica antes, el modo se identificará claramente como `eight-ball-casual` con diferencias enumeradas.

### 9.2. Elementos y objetivo

Se utiliza una bola blanca y quince bolas objeto numeradas: lisas `1–7`, ocho `8` y rayadas `9–15`. El objetivo es embolsar legalmente todas las bolas del grupo asignado y, después, la bola ocho mediante un tiro declarado y válido.

### 9.3. Formación inicial (*rack*)

- Quince bolas compactas en triángulo.
- Bola de vértice en el punto de pie (*foot spot*).
- Bola ocho en el centro del triángulo, inmediatamente debajo de las dos bolas de la segunda fila.
- En las dos esquinas inferiores debe haber una bola de cada grupo.
- El resto se distribuye sin patrón intencional que favorezca posiciones particulares.
- Las posiciones no deben crear superposiciones numéricas; tolerancias geométricas controladas.

### 9.4. Saque (*break*)

- Bola blanca en mano detrás de la línea de cabeza (*head string*).
- No se declara bola ni tronera durante el saque.
- Si se embolsa una bola sin falta, el jugador continúa, pero la mesa sigue abierta.
- Si no se embolsa ninguna bola objeto, al menos cuatro deben tocar banda para que el saque sea legal.
- Un saque ilegal habilita las opciones reglamentarias correspondientes para el jugador entrante (aceptar posición, volver a colocar y sacar, o devolver saque; consultar sección WPA 4.3).
- La bola ocho embolsada durante el saque **no es automáticamente una victoria**; su tratamiento depende de si el saque fue legal y de las opciones reglamentarias para colocarla o repetir rack.
- Un *scratch* u otras faltas de saque se resuelven según las opciones específicas del reglamento, no mediante la regla simplificada general de bola en mano sin más.

### 9.5. Mesa abierta y elección de grupo

La mesa empieza abierta. La asignación no ocurre necesariamente por embolsar cualquier bola durante el saque. Tras el saque, los grupos se asignan cuando un jugador ejecuta y consigue legalmente una bola declarada de un grupo. Hasta entonces, la mesa sigue abierta. Durante mesa abierta, la bola ocho no se golpea primero salvo la excepción reglamentaria específica de grupo completado.

### 9.6. Tiros declarados y seguridad

Fuera del saque, el jugador declara la bola que pretende introducir y la tronera elegida, conforme a las reglas aplicables. Puede declarar un tiro de seguridad (*safety*): pasa el turno al contrario al finalizar el tiro y las bolas objeto embolsadas permanecen embolsadas. El motor debe conservar esta intención **antes** de iniciar la simulación para evitar reinterpretaciones posteriores.

### 9.7. Continuación del turno

El jugador mantiene turno mientras embolse legalmente bolas de su grupo según la llamada; cuando deja de hacerlo, o tras falta, se pasa al contrario salvo excepciones específicas. El cambio de turno sucede **después de que todas las bolas se detengan** y se resuelva el evento del tiro.

### 9.8. Faltas relevantes que CUE debe detectar

- Blanca embolsada o fuera de la mesa.
- Primera bola golpeada incorrecta cuando hay grupos asignados.
- Tras el primer contacto legal, ninguna bola embolsada y ninguna bola toca banda.
- Bola objeto expulsada de la superficie de juego.
- Tiro realizado con bolas todavía en movimiento.
- Colocación ilegal de bola blanca cuando haya restricciones.
- Jugar fuera de turno, si los controles lo permiten.
- Golpes dobles o empujes ilegales, en la medida que el modelo de interacción pueda representarlos.

Faltas humanas como levantar el pie, tocar accidentalmente bolas fuera del control digital o conducta antideportiva no siempre son representables automáticamente. Se deben clasificar como **no aplicables a controles digitales** o como **decisión manual de los jugadores**, sin afirmar cobertura normativa completa.

### 9.9. Bola en mano

Tras una falta estándar durante el juego ordinario, el oponente recibe la blanca en mano y puede situarla en una zona válida de la mesa, respetando restricciones y casos especiales del saque. La herramienta de colocación debe impedir solapamientos con otras bolas y posiciones dentro de bandas o troneras.

### 9.10. Pérdida de partida

Fuera de las excepciones del saque, el jugador pierde si embolsa la ocho cometiendo falta, antes de despejar su grupo, en una tronera no declarada o si la expulsa de la mesa. La victoria exige que el grupo esté completado y la ocho se embolse legalmente en la tronera anunciada.

### 9.11. Empates y excepciones

Los casos de estancamiento, repetición de rack, bola ocho en saque, situaciones de contacto simultáneo o ambiguo, y decisiones arbitrales deben llevar pruebas específicas. Ante ambigüedad numérica, se registrarán los contactos con marcas temporales de simulación y se aplicará una política documentada; no se utilizará el orden accidental de callbacks gráficos.

### 9.12. Arquitectura normativa

Las reglas se implementarán mediante **perfiles versionados**: `free-play`, `eight-ball-casual` y `eight-ball-wpa` (este último solo si cumple el reglamento). No se mezclan reglas casuales y oficiales de manera silenciosa.

El módulo de reglas recibe un `ShotReport` inmutable y devuelve una `Ruling` (validez, faltas, bolas que siguen en mesa, turno, bola en mano, victoria/derrota y mensajes). **No debe depender de Bevy, Rapier, sonido ni cámara.**

---
## 10. Controles e interacción

### 10.1. Requisitos transversales

El jugador siempre debe comprender: **hacia dónde apunta, con qué fuerza golpeará, cuándo puede disparar y qué ha sucedido**. El sistema no debe disparar accidentalmente al seleccionar un botón, hacer scroll ni cambiar el tamaño de la ventana. Los gestos solo interceptan eventos cuando la mesa está activa y tiene foco/interacción.

### 10.2. Escritorio — propuesta de control

| Acción | Entrada primaria propuesta | Alternativa |
|---|---|---|
| Apuntar | Mover puntero alrededor de la blanca / arrastrar taco | Flechas izquierda/derecha |
| Ajustar fuerza | Arrastre controlado o barra contextual | Flechas arriba/abajo |
| Ejecutar golpe | Soltar gesto confirmado o botón «Golpear» | Espacio/Enter con foco adecuado |
| Seleccionar efecto | Selector 2D sobre la blanca | Control de dos ejes con teclado |
| Mover cámara | Función opcional distinta del apuntado | Atajos documentados |
| Pausar | Botón visible | Escape cuando no interfiera con el navegador |
| Reiniciar | Botón con confirmación | Atajo no destructivo opcional |
| Pantalla completa | Botón explícito | Ninguna dependencia de fullscreen |
| Bola/tronera declarada | Panel contextual accesible | Navegación Tab + Enter |

Los gestos exactos se someterán a pruebas de usabilidad. **No se debe forzar un control «pull back and release» si provoca tiros involuntarios.** La interacción preferida debe poder completarse también mediante un botón separado para golpear.

### 10.3. Móvil y tablet

- Eventos Pointer cuando sea posible; manejar touch/pointer de forma coherente, sin registrar duplicados.
- Zona táctil de apuntado que no tape la trayectoria principal; límites de arrastre proporcionales a la pantalla.
- Barra de potencia suficientemente grande para dedo y separada del gesto de cámara.
- Taco y guía visibles mientras se arrastra; indicador de fuerza durante preparación.
- Bloqueo opcional del giro de cámara durante el disparo.
- Soporte para orientación horizontal como recomendación, pero no dependencia exclusiva de ella.
- Adaptación a *safe areas*, notch, barras del sistema y navegador móvil.
- No usar hover como única forma de descubrir acciones.
- Prevenir desplazamiento/zoom accidental **solo en el área activa**, sin inutilizar el resto del portfolio.
- Interacciones con objetivo táctil suficiente y controles de accesibilidad externos al canvas.

### 10.4. Máquina de estados de interacción

```text
UNINITIALIZED
      ↓
   LOADING ────────────────→ ERROR / UNSUPPORTED
      ↓
    READY
      ↓
   AIMING ←─────────────────────┐
      ↓                         │
 ADJUSTING_POWER               │
      ↓                         │
 COMMITTING_SHOT                │
      ↓                         │
 BALLS_MOVING                   │
      ↓                         │
 RESOLVING_SHOT                 │
      ├──→ NEXT_TURN ───────────┤
      ├──→ BALL_IN_HAND ────────┤
      └──→ GAME_OVER

Desde READY/AIMING/NEXT_TURN: ↔ PAUSED
Desde cualquier estado montado: → DESTROYED
```

No se aceptan golpes cuando hay bolas desplazándose o el motor está pausado. La resolución de reglas se realiza **una sola vez por tiro**. Una nueva partida descarta eventos antiguos de forma controlada.

### 10.5. Asistencias al jugador

Opciones configurables:

- Línea primaria de dirección (opcional).
- Sombra o marcador de punto de contacto previsto (opcional).
- Indicador de fuerza antes del tiro.
- Ayuda de apuntado con grados de precisión: `off`, `basic` y `assisted`.
- Visualización del punto de efecto vertical/lateral.

La trayectoria asistida es **una estimación**, no una promesa exacta: debe indicar que las colisiones encadenadas, spin y rebotes pueden alterar el resultado. Desactivarla en modalidades futuras de desafío que requieran igualdad de condiciones.

### 10.6. Cámara

Cámara principal estable para apuntar, con vista elevada y profundidad suficiente para ver el volumen de las bolas. Cualquier cámara secundaria o animación cinemática debe poder desactivarse, no interrumpir el control y no cambiar la dirección de tiro calculada.

El espacio de puntero se convierte a coordenadas de mesa mediante intersección de rayo/cámara con el plano de juego. Deben controlarse relación de aspecto, escala del canvas, `devicePixelRatio` y cambios de tamaño. No proyectar directamente píxeles a metros con una constante fija.

## 11. Física y simulación

### 11.1. Estrategia elegida

El **primer prototipo usará Rapier 3D** integrado con Bevy mediante `bevy_rapier3d`. Es una corrección deliberada respecto a la idea inicial de usar un simulador 2D: aunque las bolas se trasladan mayormente sobre el plano, su rotación real ocurre en tres ejes y determina deslizamiento, rodadura y efecto. Rapier 3D permite modelar esferas rígidas y contacto con el paño, aunque **no resuelve por sí solo toda la física especial del billar**.

Si las pruebas muestran errores de rodadura, spin o rebote imposibles de corregir razonablemente, se mantiene Rapier para colisiones generales o el prototipo y se sustituye el núcleo físico por un solucionador propio especializado, preservando los contratos de CUE.

### 11.2. Unidades y coordenadas

- Unidades de mundo: **metros, kilogramos, segundos**.
- Mesa: plano horizontal `X/Z`; elevación `Y`.
- Gravedad: vector vertical negativo en `Y` cuando se emplee dinámica 3D.
- Transform visual = posición + orientación calculadas por la simulación.
- Velocidades: m/s; angular: rad/s; fuerzas: N; impulso: N·s.
- Identificador de bola estable distinto del identificador interno de entidad Bevy.
- Coordenadas normalizadas de interacción separadas de las coordenadas físicas para que las API no dependan del tamaño de la mesa.

### 11.3. Geometría física inicial de referencia

Parámetros iniciales **configurables, no constantes inmutables**:

| Parámetro | Valor de partida aproximado | Observación |
|---|---:|---|
| Superficie jugable | 2,54 m × 1,27 m | Tamaño de referencia de mesa de 9 pies; verificar contra especificación adoptada |
| Diámetro de bola | 0,05715 m | Referencia de pool estándar (2,25 pulgadas) |
| Radio de bola | 0,028575 m | Derivado del diámetro |
| Masa de bola | ≈0,17 kg | Calibración inicial; no imponer como valor reglamentario fijo |
| Número de troneras | 6 | 4 esquinas y 2 laterales |
| Paso de física inicial | 1/120 s | Hipótesis a evaluar; valorar 1/60 con subpasos |
| Render de referencia | Adaptado al dispositivo | Desacoplado del número de pasos físicos |

Las medidas de troneras, perfiles de banda, labios y vértices deberán validarse contra una especificación de mesa concreta. Un modelo visual bonito no puede compensar colliders mal posicionados.

### 11.4. Objetos simulados

**Bolas:** cuerpos rígidos dinámicos esféricos, radios y masas definidos, propiedades de contacto calibrables, velocidad lineal y angular, estado de juego `on_table`/`pocketing`/`pocketed`/`off_table`. **Mesa/paño:** collider estático horizontal. **Bandas:** colliders estáticos segmentados que reproduzcan rebotes, incluidos perfiles cercanos a troneras. **Troneras:** zonas de entrada y caída/retirada que no teletransporten prematuramente la bola. **Taco:** representación visual y lógica de golpe; no se exige colisión física continua entre malla de taco y blanca para ejecutar un tiro estándar.

### 11.5. Golpe de taco

El input del jugador produce una orden controlada:

`strike = { direction, power, impact_offset_x, impact_offset_y, elevation? }`

El núcleo valida esa orden y la transforma en velocidad/impulso lineal y, cuando proceda, torque/velocidad angular. La aplicación del impulso debe ser **única**, en la blanca, en un punto de contacto físicamente razonable. Se evita basar la potencia directamente en la duración del fotograma o en el número de eventos de puntero.

- `direction`: vector normalizado en el plano, salvo tiro especial.
- `power`: valor `0..1`, traducido a impulso mediante curva calibrada y límites máximos.
- `impact_offset_x`: desplazamiento lateral normalizado para efecto izquierda/derecha.
- `impact_offset_y`: desplazamiento vertical normalizado para follow/draw dentro de un disco permitido.
- `elevation`: reservado inicialmente; golpes de salto y masé no están comprometidos en v1.
- El golpe puede animar el taco antes de aplicarse, pero la física no debe depender del tiempo de esa animación.

### 11.6. Fenómenos físicos a contemplar

1. **Choques bola-bola:** conservación aproximada de momento e impulsos normales/tangenciales según contactos.
2. **Choques bola-banda:** restitución dependiente de normal, ángulo y condiciones del perfil.
3. **Deslizamiento a rodadura:** inicialmente la blanca puede deslizar y luego adquirir rodadura.
4. **Resistencia del paño:** frenado estable de velocidad lineal y angular; jamás usar un coeficiente mágico que cambie con los FPS.
5. **Efecto lateral:** respuesta diferenciada en bandas y contactos cuando esté habilitado.
6. **Follow y draw:** transferencia entre rotación y traslación después del choque.
7. **Contactos simultáneos o casi simultáneos:** tolerancias y orden de resolución controlados.
8. **Detección de bolas detenidas:** umbrales consistentes de traslación **y** rotación durante una ventana de tiempo; no basta verificar solo velocidad lineal cero.
9. **Embolsado:** detección geométrica con región de captación y confirmación, evitando falsos positivos cuando la bola roza la boca de la tronera.
10. **Túnel de colisión:** evaluar detección continua (*CCD*) para bolas rápidas durante el saque.
11. **Evitar inestabilidad:** no permitir que bolas atraviesen bandas, atraviesen el suelo ni se queden vibrando indefinidamente.

### 11.7. Configuración del integrador

- Ejecutar la física a **paso fijo** (`FixedUpdate` + `TimestepMode::Fixed` o equivalente validado en la versión bloqueada).
- Punto de partida: `dt = 1/120 s`, **por validar** frente a coste y estabilidad. Alternativa: `1/60 s` y dos subpasos, también por comparar.
- Renderizar con interpolación visual si es necesario, sin modificar el estado autoritativo.
- Establecer número máximo razonable de pasos de recuperación por frame para evitar espirales de bloqueo.
- Pausar la simulación al ocultar la pestaña o al desmontar el juego; reanudar sin intentar simular un salto enorme de tiempo.
- Registrar parámetros y versión del motor usados en los tests y replays.

La documentación oficial de Rapier explica que el modo variable depende de los FPS y recomienda modo fijo para obtener resultados consistentes: <https://rapier.rs/docs/user_guides/bevy_plugin/integration_parameters/>.

### 11.8. Determinismo y replays

Objetivo: **reproducibilidad controlada**, no afirmación de determinismo bit a bit entre todos los dispositivos. Para ello:

- Orden estable de creación de entidades físicas.
- Entrada de tiro cuantizada o serializada de forma inequívoca.
- Perfil físico y paso temporal versionados.
- Semilla explícita para aleatoriedad (rack/desafíos).
- Registro de tiros, estados iniciales y eventos de resultado.
- Tests comparando posiciones/velocidades dentro de tolerancias.
- Reproducibilidad exacta interplataforma solo si se verifican las condiciones exigidas por dependencias, funciones matemáticas y toolchain.

### 11.9. Detección de contactos y eventos

El simulador registrará internamente: primer contacto de la blanca, contactos posteriores de objetos con bandas, bolsillos, expulsiones y final de movimiento. Cada evento llevará `shot_id`, `sim_step`, entidad lógica y tipo. Los eventos repetidos por mantener contacto **no** deben contarse como nuevos golpes válidos.

Un `ShotReport` contiene una secuencia cronológica reducida suficiente para evaluar reglas. Esto evita que el módulo de reglas tenga que inspeccionar directamente callbacks de Rapier o mallas de Bevy.

### 11.10. Calibración física

Crear una batería de pruebas y ajustar:

- Tiro frontal de blanca contra bola inmóvil.
- Tiro rasante a diferentes velocidades.
- Dos bolas en choque oblicuo.
- Rebote de banda a múltiples ángulos.
- Spin lateral y contacto con banda.
- Follow/draw posterior a choque.
- Break con bolas compactas.
- Entrada lateral/esquina de tronera, con acierto y rechazo.
- Movimiento a 30/60/120 FPS de render: resultado final aproximadamente igual.

La referencia empírica ideal será vídeo de mesa real medible o parámetros publicados de confianza, respetando las tolerancias definidas. Sin esa calibración, solo se podrá afirmar **física convincente de videojuego**, no réplica de nivel competitivo.

### 11.11. Reglas de seguridad del estado físico

En cualquier momento deben cumplirse invariantes: no hay dos bolas con el mismo identificador; no existen coordenadas `NaN`/infinitas; el jugador solo dispara si la blanca está disponible; una bola `pocketed` no sigue generando colisiones; los cuerpos se eliminan una sola vez; la restauración de un snapshot no duplica entidades; la posición de blanca en mano siempre es válida.

## 12. Gráficos, escena y recursos

### 12.1. Renderizado

Bevy renderiza la escena 3D con una cámara apropiada para una mesa vista desde arriba. La primera ruta web se orientará a **WebGL2** por compatibilidad práctica; WebGPU se evaluará de forma separada. La selección del backend debe realizarse a nivel de configuración/build compatible, **no suponiendo que todos los navegadores admiten un fallback automático dentro del mismo build**.

### 12.2. Objetos gráficos

- Mesa: paño, bandas, madera/estructura, puntos marcados y bocas de tronera.
- Bolas: material por número/grupo, letras/números legibles, franjas fieles y rotación visible.
- Blanca: material distintivo, sin efectos que confundan punto de impacto.
- Taco: geometría elegante, punta distinguible, movimiento breve de golpe.
- Guías: líneas finas, marcador de intención, potencia y selección de bola/tronera.
- Feedback: iluminación puntual de selección, indicadores de turno y pequeñas transiciones.

### 12.3. Requisitos de assets

- Geometrías propias o assets con permisos compatibles con publicación open source.
- Texturas comprimidas y adaptadas al uso, sin archivos gigantes innecesarios.
- Modelos 3D optimizados; materiales reutilizados entre objetos.
- Numeración y bandas correctas; tamaños relativos coherentes.
- Separación estricta entre geometría renderizada y collider físico simplificado/calibrado.
- Inventario de origen, autor, licencia y cambios de **cada recurso**.
- Prohibición de incorporar recursos extraídos de otros videojuegos sin autorización.

### 12.4. LOD y optimización

Para bolas y mesa, usar geometrías razonables y seleccionar nivel de detalle/calidad según presupuesto. Evitar sombras caras, reflejos dinámicos innecesarios o postprocesado pesado en móviles. Precalcular lo estático cuando sea viable. Medir *draw calls*, número de triángulos, memoria de texturas y tiempo GPU.

### 12.5. Gestión de cámara y canvas

- El canvas pertenece a la instancia CUE, dentro del contenedor indicado por el host.
- Escuchar `ResizeObserver` o mecanismo equivalente para reajustar proyección.
- Respetar `devicePixelRatio` con límite configurable para evitar resoluciones absurdas.
- Reconstruir/reinicializar recursos tras pérdida de contexto gráfico cuando sea viable; si no, mostrar error recuperable.
- No fijar tamaños absolutos que rompan el diseño responsivo.
- No usar una segunda escena Bevy por cada panel de UI si basta HTML accesible superpuesto.

## 13. Audio y respuesta sensorial

### 13.1. Sonidos requeridos

- Taco contra blanca.
- Bola contra bola.
- Bola contra banda.
- Bola al entrar en tronera.
- Confirmación de golpe/listo.
- Feedback discreto de falta, turno y victoria.

### 13.2. Comportamiento acústico

Los sonidos deberán variar moderadamente según impulso de choque, sin saturación ni repeticiones monótonas. Evitar reproducir cada frame del contacto continuo. La mezcla deberá priorizar eventos de juego sobre música o elementos decorativos. Música de fondo **no requerida**.

### 13.3. Restricciones de navegador

El audio **solo se inicializa o reanuda tras interacción autorizada del usuario**, conforme a las restricciones habituales de autoplay. Al pausar/silenciar se interrumpe o ajusta correctamente el audio. Los sonidos deberán funcionar sin obligar a activar el sonido: todo resultado importante también aparecerá visualmente.

Fuente: <https://developer.mozilla.org/en-US/docs/Web/Media/Guides/Autoplay>.

### 13.4. Configuración

`audio.enabled`, `master_volume`, `effects_volume` y preferencias locales. Si se almacenan preferencias en el navegador, documentar dónde se guardan, cómo se reinician y no confundir persistencia local con sincronización remota.

## 14. Stack tecnológico completo

### 14.1. Núcleo principal

| Capa | Herramienta | Versión de referencia | Papel |
|---|---|---|---|
| Lenguaje | Rust estable | Bloquear versión de toolchain al iniciar | Lógica, simulación, reglas, integración del motor |
| Gestión de proyecto | Cargo y Rustup | Correspondientes al toolchain | Dependencias, builds, tests y *workspace* |
| Motor de juego | **Bevy** | **0.20** | ECS, escena, cámara, render, input, scheduling y assets |
| Física | **`bevy_rapier3d`** | **0.37.0** | Puente Bevy–Rapier para cuerpos, contactos y colisiones 3D |
| Motor físico indirecto | `rapier3d` | Compatibilidad transitiva fijada por plugin | Simulación de cuerpos rígidos |
| Destino web | `wasm32-unknown-unknown` | Toolchain fijado | Compilación a WebAssembly |
| Puente WASM/web | `wasm-bindgen` | Versión compatible bloqueada | Glue JS, llamadas hacia/desde WASM |
| Web de integración | JavaScript estándar, módulos ES | Navegadores soportados | Carga, ciclo de vida, eventos y montaje |
| Gráficos web | WebGL2 preferido; WebGPU opcional | Según Bevy/navegador | Backend GPU de la escena |
| Distribución web | Archivos estáticos (`.wasm`, `.js`, assets) | Versionados | Hosting sin API de servidor |

**Compatibilidad verificada a la fecha:** el paquete `bevy_rapier3d` 0.37.0 declara dependencia `bevy ^0.20`. Véase <https://docs.rs/crate/bevy_rapier3d/0.37.0>. La versión no debe elevarse unilateralmente sin verificar plugin, rendimiento web y breaking changes. Bevy 0.20 fue anunciado el 8 de octubre de 2026: <https://bevy.org/news/bevy-0-20/>.

### 14.2. Bibliotecas complementarias propuestas

| Herramienta | Uso | ¿Obligatoria? |
|---|---|---|
| `serde` / `serde_json` | Serialización de configuración, snapshots y perfiles | Probable |
| `thiserror` | Tipos de error explícitos | Recomendada |
| `tracing` o `log` | Eventos de desarrollo y diagnósticos | Recomendada |
| `rand` o equivalente | Semillas/racks/challenges | Solo cuando exista aleatoriedad |
| `proptest` | Tests basados en propiedades | Recomendada |
| `insta` o snapshots equivalentes | Validación de reportes/estados | Opcional |
| `cargo clippy` | Análisis estático Rust | Obligatoria en CI |
| `rustfmt` | Formato Rust | Obligatoria en CI |
| `cargo audit` / alternativa mantenida | Revisión de vulnerabilidades de crates | Recomendada |
| `cargo deny` | Licencias, duplicados y procedencia | Recomendada |
| Playwright u otra herramienta de navegador | E2E de integración y controles | Recomendada |
| Vite u otro servidor de ejemplo | Servir host de demostración; **no motor** | Opcional |
| `wasm-opt` / Binaryen | Optimización y verificación de WASM | Tras comprobar beneficios |

No introducir librerías que agreguen complejidad sin un requisito concreto. Evitar instalar un frontend completo solo para representar botones de menú.

### 14.3. Lo que NO forma parte del stack base

Node.js/TypeScript como lenguaje del juego; React como dependencia obligatoria; Next.js como requisito; servidor Rust/Axum/Actix; PostgreSQL; Redis; Firebase; autenticación OAuth; sockets multijugador; servicios de pago; APIs privadas; proveedores analíticos por defecto.

> **Aclaración:** puede usarse Node.js para ejecutar un servidor de desarrollo, empaquetar JS o automatizar pruebas de navegador, pero ello **no convierte CUE en un proyecto TypeScript**. El código del juego sigue en Rust.

### 14.4. Bevy vs Rapier: responsabilidades

Bevy administra entidades, componentes, scheduling, cámaras, materiales, escenas y presentación. Rapier calcula la dinámica y los contactos. CUE Core define acciones, reglas, estados, reportes y calibraciones específicas del billar. Ninguna de estas tres capas debe asumir competencias propias de las otras.

### 14.5. Elección consciente de Rapier 3D

`bevy_rapier2d` no se descarta como opción técnica para prototipos planos, pero para el objetivo declarado de efectos, rotación y colisiones tridimensionales resulta más congruente empezar con `bevy_rapier3d`. Esta elección debe verificarse con el *spike* de simulación; no se mantendrá por orgullo tecnológico si degrada la experiencia.

## 15. Arquitectura de software

### 15.1. Diagrama lógico

```text
                   ┌──────────────────────────────┐
                   │  PORTFOLIO / SITIO ANFITRIÓN │
                   │     HTML, CSS, router, etc.  │
                   └──────────────┬───────────────┘
                                  │ API JS mínima
                   ┌──────────────▼───────────────┐
                   │         CUE WEB BRIDGE       │
                   │ load · mount · pause · destroy│
                   │  eventos · config · errores  │
                   └──────────────┬───────────────┘
                                  │ wasm-bindgen
                   ┌──────────────▼───────────────┐
                   │           CUE APP            │
                   │      Bevy · render · UI      │
                   │       input · audio          │
                   └───────────┬───────┬──────────┘
                               │       │
                ┌──────────────▼─┐ ┌───▼────────────────┐
                │   CUE CORE     │ │   CUE PHYSICS      │
                │ reglas/estados │ │ Rapier 3D/adaptador│
                │ acciones/event│ │ impactos/movimiento│
                └──────────────┬─┘ └───┬────────────────┘
                               └───────┤
                                       ▼
                          STATE / SHOT REPORT
```

### 15.2. Capas

**A. `cue-core`**  
Rust sin Bevy ni dependencias de navegador. Define tipos estables, reglas, estados, reportes, validaciones, configuración lógica y evaluación de resultados. Debe compilar y probarse de forma nativa y, en la medida apropiada, en WASM.

**B. `cue-physics`**  
Adaptador de simulación y parámetros de billar. Gestiona entidades físicas, colisiones, impulsos, medición y generación de eventos de tiro. Inicialmente depende de Rapier y puede utilizar APIs Bevy a través de un adaptador separado; **las reglas no dependen de él**.

**C. `cue-game`**  
Aplicación Bevy que une input, estado, física, render, recursos, audio y lógica de las pantallas. Orquesta transiciones sin reinterpretar decisiones reglamentarias.

**D. `cue-web`**  
Código mínimo de interoperabilidad Rust↔JS, montaje en contenedor, exposición de métodos, eventos y liberación. Esta capa no debe asumir la estructura de rutas ni los estilos globales del portfolio.

**E. `cue-example`**  
Host web independiente usado como referencia pública de integración. Nunca depender de importar código interno del portfolio personal.

### 15.3. Regla de dependencias

```text
cue-core          → Rust estándar + utilidades puras
cue-physics       → contratos de cue-core + Rapier
cue-game          → cue-core + cue-physics + Bevy
cue-web           → cue-game + wasm-bindgen/web APIs
portfolio         → paquete web publicado / assets de CUE
```

No permitir ciclos. Un adaptador que una Bevy con Rapier puede residir en `cue-game` si resulta imposible mantener `cue-physics` completamente agnóstico; la **frontera pública del core** permanece pura.

### 15.4. ECS de Bevy

Propuesta de componentes: `BallId`, `BallVisual`, `BallPhysicsRef`, `PocketId`, `TablePart`, `CueStick`, `ShotMarker`, `AimGuide`, `GameEntity`. Recursos: `CueConfiguration`, `GameSession`, `InputIntent`, `PhysicsCalibration`, `AudioSettings`, `RenderSettings` y `Diagnostics`. Sistemas: `handle_input`, `validate_intent`, `commit_shot`, `advance_physics`, `collect_physics_events`, `resolve_shot`, `apply_ruling`, `sync_visuals`, `emit_public_events`, `pause_resume`, `cleanup`.

Se debe definir su **orden de ejecución**. Un evento visual nunca debe disparar por sí solo una segunda resolución reglamentaria.

### 15.5. Principios de independencia

- Las reglas funcionan sin GPU.
- La física puede ejecutarse en entorno de tests con escena mínima.
- Los sonidos no influyen en velocidades.
- Los efectos visuales no alteran colisionadores.
- La configuración del portfolio no modifica el estado interno de CUE sin una API declarada.
- El contrato público no publica referencias directas a entidades Bevy como identificadores persistentes.
- Se evita usar singletons globales en JS que impidan desmontar y montar limpiamente.

## 16. Modelo de datos, estados y eventos

### 16.1. Entidades de dominio

| Entidad | Campos mínimos propuestos |
|---|---|
| `Ball` | `id`, `kind`, `group?`, `position`, `orientation`, `linear_velocity`, `angular_velocity`, `status` |
| `TableSpec` | `playfield_length`, `playfield_width`, `cushion_geometry`, `pocket_geometry`, `cloth_profile` |
| `CueStrike` | `direction`, `power`, `impact_offset`, `shot_id` |
| `Player` | `id`, `display_name?`, `group?`, `is_active` |
| `GameSession` | `mode`, `phase`, `current_player`, `rack_number`, `score`, `configuration_version` |
| `ShotReport` | `shot_id`, `first_object_hit`, `pocketed`, `rail_contacts`, `cue_scratch`, `balls_off_table`, `events` |
| `Ruling` | `legal`, `fouls`, `next_player`, `ball_in_hand`, `victory`, `loss`, `message_key` |
| `CueSettings` | `theme`, `audio`, `assists`, `quality`, `controls`, `camera` |

### 16.2. Identidad y versionado

Las entidades poseen IDs lógicos determinables (`CueBall`, `ObjectBall(1..=15)`), no IDs ECS expuestos al público. Los formatos serializados llevan `schema_version`. Cada resultado conoce su `ruleset_id` y `physics_profile_id`. Un archivo antiguo o un enlace de desafío incompatible debe rechazarse limpiamente o migrarse mediante transformaciones explícitas.

### 16.3. Estados de partida

`uninitialized` → `ready` → `awaiting_shot` → `simulating` → `resolving` → `awaiting_shot` / `ball_in_hand` / `finished`; `paused` suspende transitoriamente; `disposed` es terminal para esa instancia. Se debe separar el estado de sesión del estado de carga gráfica.

### 16.4. Eventos internos y públicos

| Evento público | Cuándo se emite | Información |
|---|---|---|
| `ready` | Juego inicializado y operable | versión/build |
| `shot-start` | Golpe validado | `shotId`, jugador |
| `ball-pocketed` | Embolsado confirmado | `ballId`, `pocketId` |
| `shot-end` | Todas las bolas detenidas y reporte cerrado | `ShotReport` resumido |
| `foul` | Regla detecta falta | `foulCodes`, jugador |
| `turn-changed` | Turno reasignado | jugador anterior/nuevo |
| `game-over` | Victoria/derrota confirmada | ganador y razón |
| `paused` / `resumed` | Cambio de ejecución | razón |
| `error` | Fallo recuperable o fatal | código y descripción segura |
| `destroyed` | Recursos liberados | sin datos personales |

En un tiro pueden generarse varios `ball-pocketed` pero **un solo `shot-start` y un solo `shot-end`**. Todos los eventos usan nombre y formato versionables.

### 16.5. Validaciones importantes

No aceptar valores fuera de rango, `NaN`, posiciones solapadas, fuerza negativa, modos inexistentes, configuración con dimensiones cero, colores inválidos o callbacks que se ejecuten tras destrucción. Las entradas del host son datos no confiables aunque el juego no use backend.

## 17. API pública y contrato de integración

### 17.1. Objetivo

Integrar CUE con aproximadamente una llamada de inicialización y un contenedor HTML, sin exigir conocimientos de Rust ni dependencia obligatoria de framework. Se recomienda distribuir **un wrapper ESM y assets WASM**; el código Rust puede exponerse adicionalmente como crates.

### 17.2. Interfaz conceptual de JavaScript

```js
// API objetivo: ejemplo ilustrativo, no código publicado actualmente.
import { createCue } from '@tuletta/cue';

const root = document.querySelector('#cue-root');

const game = await createCue({
  container: root,
  mode: 'free-play',
  theme: 'studio',
  quality: 'auto',
  audio: { enabled: true, volume: 0.65 },
  controls: { keyboard: true, pointer: true, touch: true },
  assists: 'basic',
});

game.on('ready', () => console.info('CUE disponible'));
game.on('shot-end', (event) => console.info(event));
game.on('error', (event) => console.error(event.code));

await game.start();

// Al navegar fuera o desmontar el componente:
await game.destroy();
```

`@tuletta/cue` es **nombre provisional sujeto a disponibilidad y decisión editorial**. El método de importar assets puede variar según bundler. No se publicará la firma definitiva sin probarla en al menos un host Vite y un host con JavaScript sin framework.

### 17.3. Contrato mínimo de instancia

| Método | Semántica |
|---|---|
| `start()` | Inicia sesión si aún no se ha iniciado; no crea instancia duplicada |
| `pause()` | Detiene simulación y gestos sin destruir render/estado |
| `resume()` | Continúa una sesión pausada |
| `reset()` | Reinicia el modo actual con confirmación a cargo del host o UI |
| `setMode(mode)` | Reinicia/cambia modalidad mediante transición válida |
| `setTheme(theme)` | Aplica tema visual sin alterar físicas |
| `setAudio(settings)` | Ajusta o silencia sonido |
| `resize()` | Recalcula dimensiones cuando sea necesario |
| `on(event, handler)` | Suscripción tipada, con forma de cancelar la suscripción |
| `getState()` | Snapshot público de solo lectura, no referencias vivas a ECS |
| `destroy()` | Cierra recursos; idempotente y seguro aunque se invoque dos veces |

`destroy()` debe terminar animación/render, simulación, carga pendiente, `ResizeObserver`, eventos globales, tareas, audio y referencias a canvas en la medida garantizada por las APIs actuales. El comportamiento de nuevas llamadas después de destruir debe estar documentado (preferiblemente error `INSTANCE_DESTROYED`).

### 17.4. Configuración pública sugerida

```text
CueOptions
├─ container: HTMLElement                  REQUIRED
├─ mode: 'free-play' | 'eight-ball-local'
├─ theme: 'classic' | 'studio' | 'contrast'
├─ quality: 'auto' | 'low' | 'medium' | 'high'
├─ render: { pixelRatioCap?, preferWebGPU? }
├─ physics: { preset? }                  RESTRINGIDO / no tuning arbitrario en 8-Ball
├─ audio: { enabled, volume }
├─ controls: { pointer, touch, keyboard, sensitivity? }
├─ assists: 'off' | 'basic' | 'assisted'
├─ locale: 'es' | 'en' | ...
├─ accessibility: { reducedMotion?, highContrast? }
└─ callbacks / event subscriptions
```

No permitir que el host cambie parámetros normativos a mitad de un tiro o de una partida competitiva. Aplicar cambios de perfil físico solamente en reinicios/sesiones nuevas, con validación.

### 17.5. Bridge WASM

`wasm-bindgen` o el proceso equivalente recomendado por Bevy generará módulos JavaScript de interoperabilidad a partir del binario `.wasm`. La capa JS administrará inicialización asincrónica, importación de assets y errores; la lógica del billar permanece en Rust. Documentación técnica: <https://wasm-bindgen.github.io/wasm-bindgen/>.

**Advertencia arquitectónica:** empaquetar una aplicación Bevy completa no es idéntico a empaquetar una pequeña biblioteca Rust pura con `wasm-pack`. Para el juego Bevy deberá validarse el pipeline de construcción `cargo` + `wasm-bindgen` + assets; `wasm-pack` puede resultar útil para crates WASM aislados, pero **no se asumirá que el comando básico crea automáticamente un paquete Bevy listo para npm**.

### 17.6. Límite de instancias

La v1 soportará **una instancia activa por página** si Bevy y el bridge no permiten una gestión fiable de múltiples aplicaciones/canvas en el mismo runtime. El wrapper detectará intentos simultáneos y responderá con error explícito en lugar de fallar silenciosamente. Multiinstancia será una capacidad futura supeditada a pruebas de aislamiento.

### 17.7. Integraciones adicionales

Los wrappers React, Vue y Svelte serán capas delgadas sobre la API JS, no versiones distintas del motor. Deben reflejar el ciclo de vida propio de cada framework y evitar dobles inicializaciones durante el desarrollo. La primera publicación no necesita obligatoriamente todos esos wrappers.

## 18. Integración en el portfolio

### 18.1. Estructura de navegación propuesta

```text
Portfolio
├─ /
│   └─ Tarjeta/preview liviano del proyecto CUE
├─ /projects
│   └─ Información, capacidades y enlace al código
└─ /cue
    ├─ Contenedor de juego
    ├─ Capa de accesibilidad / controles HTML
    └─ Enlaces discretos: proyecto · código · salir
```

La ruta exacta debe adaptarse al router actual del portfolio. Este documento **no presupone su framework ni su estructura de repositorio**.

### 18.2. Carga bajo demanda

- La home no importa por defecto el WASM completo ni los modelos de mesa.
- El paquete se descarga al entrar a `/cue` o tras pulsar «Jugar», según mediciones.
- Puede predescargarse con baja prioridad en condiciones favorables sin bloquear el sitio.
- Mostrar progreso honesto o indicador de carga, no animación infinita sin diagnóstico.
- Si el navegador no admite el backend necesario, mostrar alternativa descriptiva y enlace al repositorio, no pantalla en blanco.

### 18.3. Independencia visual

CUE controla únicamente su raíz y estilos asociados. No modifica globalmente `body`, tipografía, overflow o atajos fuera del contexto necesario; si usa pantalla completa/restaura scroll, debe revertir cada modificación al salir. Los estilos del portfolio no deben alterar controles ni dimensiones internas de forma inesperada.

### 18.4. Ciclo de vida de navegación SPA

**Montaje:** localizar el contenedor → comprobar capacidad web → cargar recursos → crear instancia → registrar listeners → marcar estado listo. **Desmontaje:** parar input → pausar física → anular peticiones/tareas cuando proceda → liberar GPU/audio/listeners → retirar DOM propio → marcar destruido. **Remontaje:** debe comportarse como inicialización limpia, sin segundo loop de render oculto.

### 18.5. Modo pantalla completa

Activación mediante gesto explícito y Fullscreen API cuando esté disponible. Gestionar denegación/salida; la función no debe ser requisito para jugar. En iOS y otros entornos con restricciones de Fullscreen API, se ofrecerá un modo inmersivo CSS cuando sea posible sin depender de permisos especiales.

### 18.6. Comunicación con portfolio

Eventos para que el portfolio muestre información contextual, enlace GitHub o telemetría **únicamente si el host ya posee y documenta ese sistema**. CUE en sí mismo no hace solicitudes a una API de analítica. No compartir información de usuario ni de la sesión fuera de eventos de juego locales de manera predeterminada.

### 18.7. Distribución estática

El sitio puede publicar `cue.js`, `cue_bg.wasm`, shaders/assets y manifiestos en hosting estático con HTTPS. Deben configurarse MIME type `application/wasm`, caché con nombres versionados y rutas de assets coherentes. El hosting entrega archivos; **no ejecuta la simulación**.

---
## 19. Estructura del repositorio

### 19.1. Propuesta de monorepo Rust

```text
CUE/
│
├── README.md                         # Presentación, demo e instalación
├── LICENSE                           # Licencia del código (tras aprobación)
├── NOTICE.md                         # Créditos y avisos si corresponden
├── CONTRIBUTING.md                   # Guía de colaboración
├── CODE_OF_CONDUCT.md                # Normas de comunidad
├── SECURITY.md                       # Política de divulgación
├── CHANGELOG.md                      # Evolución pública
├── Cargo.toml                        # Workspace Rust
├── Cargo.lock                        # Versiones exactas (incluidas en build app)
├── rust-toolchain.toml               # Toolchain bloqueada
├── rustfmt.toml                       # Estilo
├── .gitignore
├── .github/
│   ├── workflows/
│   │   ├── rust-check.yml
│   │   ├── wasm-build.yml
│   │   ├── browser-e2e.yml
│   │   └── release.yml
│   ├── ISSUE_TEMPLATE/
│   └── PULL_REQUEST_TEMPLATE.md
│
├── crates/
│   ├── cue-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── domain.rs
│   │       ├── ball.rs
│   │       ├── table.rs
│   │       ├── game_state.rs
│   │       ├── commands.rs
│   │       ├── events.rs
│   │       ├── errors.rs
│   │       ├── serialization.rs
│   │       └── rules/
│   │           ├── mod.rs
│   │           ├── free_play.rs
│   │           ├── eight_ball.rs
│   │           └── fouls.rs
│   │
│   ├── cue-physics/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── config.rs
│   │       ├── contacts.rs
│   │       ├── cushion.rs
│   │       ├── cloth.rs
│   │       ├── cue_strike.rs
│   │       ├── pockets.rs
│   │       ├── reports.rs
│   │       └── world.rs
│   │
│   ├── cue-game/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── app.rs
│   │       ├── plugins.rs
│   │       ├── ecs.rs
│   │       ├── systems/
│   │       ├── rendering/
│   │       ├── inputs/
│   │       ├── camera/
│   │       ├── audio/
│   │       └── ui/
│   │
│   └── cue-web/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── bindings.rs
│           ├── lifecycle.rs
│           └── config.rs
│
├── web/
│   ├── package/                     # Wrapper ESM / metadata npm
│   │   ├── src/
│   │   │   ├── index.js
│   │   │   ├── loader.js
│   │   │   ├── events.js
│   │   │   └── styles.css
│   │   ├── package.json
│   │   └── README.md
│   └── example-vanilla/             # Demostración independiente
│       ├── index.html
│       ├── main.js
│       └── styles.css
│
├── assets/
│   ├── models/
│   ├── textures/
│   ├── audio/
│   ├── icons/
│   └── ATTRIBUTION.md
│
├── tests/
│   ├── physics_cases/
│   ├── rules_cases/
│   ├── integration/
│   ├── snapshots/
│   └── browser/
│
├── benchmarks/
│   ├── physics/
│   └── web/
│
├── docs/
│   ├── architecture.md
│   ├── getting-started.md
│   ├── web-integration.md
│   ├── physics-model.md
│   ├── rules-eight-ball.md
│   ├── controls.md
│   ├── accessibility.md
│   ├── browser-support.md
│   ├── asset-policy.md
│   ├── api.md
│   ├── releases.md
│   └── adr/
│       ├── 0001-rust-bevy.md
│       ├── 0002-rapier-3d.md
│       ├── 0003-wasm-webgl2.md
│       └── 0004-no-backend.md
│
└── scripts/
    ├── build-web.sh
    ├── optimize-wasm.sh
    ├── package-web.sh
    └── check-assets.sh
```

La estructura es un **mapa de responsabilidades**, no una orden de crear de inmediato archivos vacíos. Se incorporarán carpetas cuando haya código o documentación real. Si dos crates separadas no aportan una frontera útil en la práctica, consolidarlas mediante un ADR, sin sacrificar los contratos de dominio.

### 19.2. Separación respecto al portfolio

- CUE tiene su propio repositorio y versionado.
- El portfolio depende de una versión publicada o de un artefacto de build reproducible de CUE.
- El portfolio no importa módulos internos privados de `cue-game` ni replica físicamente los assets.
- Un cambio en la home del portfolio no obliga a recompilar la física.
- Una nueva versión CUE se adopta mediante actualización intencional del paquete o manifiesto.

## 20. Preparación, desarrollo y compilación

### 20.1. Entorno de desarrollo

Requisitos: Rust stable administrado por `rustup`; Cargo; Git; target de WASM; herramienta `wasm-bindgen` compatible; navegador moderno; servidor HTTP local; opcionalmente Node.js para empaquetado y pruebas. Un editor con `rust-analyzer` ofrece completado y diagnósticos.

### 20.2. Preparación básica (comandos orientativos)

```bash
rustup update stable
rustup target add wasm32-unknown-unknown

cargo --version
rustc --version

# Una vez creado el workspace:
cargo check --workspace
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

**Antes de comenzar el trabajo real**, sustituir la toolchain móvil por la versión exacta comprobada en `rust-toolchain.toml` y bloquear las versiones en el proyecto. La versión de `wasm-bindgen-cli` utilizada en CI debe ser compatible con el crate que genera las bindings; una discrepancia puede hacer fallar la carga.

### 20.3. Ejemplo conceptual de dependencias

```toml
# Fragmento de Cargo.toml del módulo que integra el motor.
# No equivale por sí solo a un juego web compilable.
[dependencies]
bevy = "0.20"
bevy_rapier3d = "0.37.0"
```

Los *features* de Bevy para web, los plugins, los assets y la configuración de linking se decidirán tras montar la prueba mínima WebAssembly. No se deben inventar combinaciones de `default-features = false` sin ejecutar el build.

### 20.4. Ciclo de desarrollo recomendado

1. Programar y probar dominio/física primero con binarios y tests nativos (iteración más rápida).
2. Conectar la escena Bevy mínima y verificar render nativo.
3. Crear un **spike web temprano**, no esperar al final del proyecto para compilar a WASM.
4. Integrar input y eventos JS↔WASM.
5. Medir rendimiento/memoria en navegador real.
6. Introducir aspectos gráficos de forma incremental.
7. Construir demo independiente y después integrar el paquete en el portfolio.

### 20.5. Ruta de compilación para navegador

El proceso validado por los ejemplos oficiales de Bevy contempla generar `.wasm` con Cargo y las bindings JS con `wasm-bindgen`. Flujo conceptual:

```bash
# 1. Compilar el crate/ejecutable web configurado para el target.
cargo build --release --target wasm32-unknown-unknown -p cue-web

# 2. Generar bindings ES para navegador.
# La ruta exacta del .wasm dependerá del tipo de crate/target final.
wasm-bindgen \
  --target web \
  --out-dir web/dist \
  target/wasm32-unknown-unknown/release/cue_web.wasm

# 3. Copiar los assets autorizados y servir web/dist mediante HTTP.
# 4. Verificar carga, input, audio y destrucción en navegador.
```

**Importante:** los comandos constituyen una **plantilla de pipeline**, no una garantía de funcionamiento inmediato. El nombre/ruta del `.wasm`, tipo de target, *features* Bevy y contenido de la salida deberán ajustarse tras generar el proyecto. Bevy documenta un flujo semejante en sus ejemplos oficiales: <https://github.com/bevyengine/bevy/blob/main/examples/README.md>.

### 20.6. Construcción del paquete web

- Conservar el binario WASM, glue JS, metadata y recursos estáticos necesarios.
- Publicar ESM y un manifiesto legible por bundlers modernos.
- Entregar una URL o ruta de `assetBase` configurable para hosts con subrutas/CDN.
- Evitar rutas absolutas codificadas en el binario o CSS.
- Generar checksum/manifiesto de los assets principales.
- Prueba de publicación con un consumidor externo que no comparte el monorepo.
- Separar versión de paquete de versión de formato de estado si evoluciona el esquema.

### 20.7. Modo desarrollo vs producción

En desarrollo se permiten diagnósticos, overlays de colliders, trazas de tiros y perfiles de físicas. En producción se desactivan renderizadores de depuración, assets sobrantes y logs masivos; se optimiza el WASM y se verifica que la compilación de release no altera las reglas.

### 20.8. Scripts previstos

| Script | Responsabilidad |
|---|---|
| `build-web` | Build Rust WASM + bindings y archivos finales |
| `check-rust` | `fmt`, `clippy`, tests del workspace |
| `test-physics` | Casos físicos con tolerancias |
| `test-rules` | Matriz de reglas WPA/casual |
| `test-browser` | E2E Chrome/Firefox/WebKit disponibles |
| `benchmark-web` | FPS, carga, memoria e interacción |
| `check-assets` | Créditos/licencias/tamaños |
| `package-web` | Artefacto apto para distribuir |

## 21. Rendimiento y compatibilidad

### 21.1. Filosofía

Un juego embebido no debe comprometer la navegación del portfolio. **El coste de CUE solo se justifica cuando la persona decide interactuar con él.** Se priorizan arranque progresivo, reducción del tamaño inicial, perfiles gráficos y pausa al salir de la vista.

### 21.2. Presupuesto de tiempo por frame

A 60 FPS hay aproximadamente **16,7 ms por fotograma**; a 30 FPS, **33,3 ms**. Son presupuestos totales que incluyen lógica, render, CPU/GPU, puente y tareas del navegador; no todo el tiempo está disponible para físicas.

Objetivos iniciales sujetos a hardware de referencia:

- Escritorio medio: alcanzar 60 FPS sostenidos con mesa completa.
- Móvil de gama media: alcanzar como mínimo 30 FPS sostenidos, sin cierres ni sobrecalentamiento evidente durante sesiones de prueba.
- Tiro con 16 bolas: no congelar controles ni producir pausas prolongadas.
- Pestaña en segundo plano: suspensión o degradación segura; sin carga de CPU/GPU significativa.
- Navegación repetida `/cue` → portfolio → `/cue`: no acumular render loops ni listeners.

### 21.3. Presupuesto de descarga

No se establecerá como promesa un tamaño de `.wasm` antes de medir una build real de Bevy. En la fase de benchmark se registrarán, por separado: tamaño comprimido y sin comprimir de WASM; glue JS; modelos; texturas; audio; descarga inicial; tiempo de compilación/instanciación; tiempo hasta primer frame y tiempo hasta tiro posible.

Aplicar: *code splitting* del portfolio, assets comprimidos, eliminación de features no usados, optimización del binario donde aporte beneficio, caché de recursos versionados y degradación gráfica en móviles. El objetivo es **reducir el coste real percibido**, no ocultarlo con una pantalla de carga elaborada.

### 21.4. Perfiles de calidad

| Perfil | Cambios permitidos | No puede cambiar |
|---|---|---|
| `low` | sombras reducidas, resolución limitada, reflejos simples | reglas, radios, colisiones, resultados físicos |
| `medium` | materiales y sombras equilibrados | precisión de input, reglas |
| `high` | mayor nitidez, detalles y efectos moderados | simulación autoritativa |
| `auto` | elige por capacidad y telemetría local de rendimiento | decisiones reglamentarias |

La selección `auto` no debe depender de rastrear al usuario ni solicitar permisos. Puede medir FPS y capacidad del navegador localmente y aplicar degradación gradual.

### 21.5. Matriz mínima de compatibilidad a validar

| Entorno | Prioridad | Resultado exigido antes de anunciar soporte |
|---|---|---|
| Chrome/Chromium actual, Windows/macOS/Linux | Alta | Carga, input, audio y cierre correctos |
| Firefox actual, escritorio | Alta | Igual que anterior o limitación publicada |
| Safari actual, macOS | Alta | Igual que anterior o limitación publicada |
| Safari iOS/iPadOS | Alta | Táctil, orientación, audio y límites de fullscreen |
| Chrome Android | Alta | Rendimiento móvil aceptable y controles táctiles |
| Navegadores antiguos/WebGL2 ausente | Degradación | Mensaje legible y sin crash |
| WebGPU | Experimental | Se anuncia por separado, nunca sustituye soporte verificado |

Definir versiones mínimas **después** del spike y registrar resultados reproducibles. No prometer soporte universal por el mero hecho de usar WebAssembly.

### 21.6. Fallos de plataforma

Gestionar `WebAssembly` no disponible, creación de contexto gráfico fallida, restricciones de autoplay, ausencia de Fullscreen API, pérdida de contexto GPU, red interrumpida, assets faltantes y memoria insuficiente. El usuario debe poder salir de la vista y el portfolio debe seguir funcionando.

## 22. Accesibilidad y experiencia responsable

### 22.1. Objetivo

Aplicar principios WCAG 2.2 a la **interfaz web que rodea el canvas** y ofrecer controles alternativos razonables. No afirmar que el juego completo es universalmente accesible sin validación con tecnologías de apoyo.

### 22.2. Requisitos específicos

- Menús, pausa, reinicio, selección de modo, ayuda, audio y bolas/troneras declaradas navegables por teclado.
- Control del tiro mediante teclado como alternativa a puntero/táctil.
- Foco visible; sin trampa de teclado y con retorno de foco al cerrar el juego.
- Nombres accesibles en los botones; mensajes de estado relevantes mediante una región apropiada para lectores de pantalla.
- Contraste suficiente en controles; no identificar grupos únicamente por su color.
- Indicación textual de turnos/faltas/victoria junto al sonido.
- Respetar `prefers-reduced-motion` y permitir reducir transiciones de cámara.
- No hacer depender el avance de interacciones cronometradas con reflejos rápidos.
- Áreas táctiles amplias y separación apropiada de botones destructivos.
- El juego debe ofrecer instrucciones breves, accesibles y consultables nuevamente.
- Todo elemento fuera del canvas debe seguir funcionando si WebGL no está disponible.

### 22.3. Límites de accesibilidad del canvas

Una malla 3D no aporta semántica HTML automáticamente. Crear una capa HTML accesible que comunique la situación: bolas restantes, turno actual, estado de disparo y resultado del tiro. Para una modalidad de orientación avanzada podrían considerarse ayudas sonoras/textuales en una fase posterior.

Referencias: <https://www.w3.org/TR/WCAG22/> y <https://www.w3.org/WAI/WCAG22/Techniques/general/G202>.

## 23. Privacidad, seguridad y operación

### 23.1. Política de privacidad por diseño

La versión base **no solicita nombre real, correo, contraseña, teléfono, ubicación, cámara, micrófono ni permisos de terceros**. No incluye registro, cookies analíticas propias ni servidor de partidas. Jugar no implica aceptar rastreo adicional por parte de CUE.

### 23.2. Persistencia local opcional

Puede almacenarse en `localStorage` únicamente: sonido activado, volumen, tema, esquema de control, asistencias y mejores marcas locales en un futuro modo de desafíos. Es necesario definir claves con prefijo y versión (`cue:v1:settings`), validar los valores, manejar excepciones de almacenamiento y permitir restablecer preferencias.

No almacenar identificadores personales. Los récords locales son **manipulables** y no deben presentarse como clasificación verificable.

### 23.3. Datos y hosting del portfolio

Que CUE no envíe datos no implica que el proveedor de hosting del portfolio no registre solicitudes HTTP. La política de privacidad de la web anfitriona debe explicar sus propias prácticas. Las métricas internas de rendimiento pueden existir solo en memoria sin transmisión.

### 23.4. Seguridad de integración

- Tratar opciones, fragmentos de URL y configuraciones como entradas no confiables.
- Validar rangos, tamaños, formatos, nombres de temas y coordenadas.
- Evitar insertar datos externos mediante `innerHTML` sin sanitización.
- No ejecutar comandos provenientes del host ni admitir rutas arbitrarias de filesystem.
- Reducir dependencias y ejecutar auditorías periódicas.
- Publicar scripts/WASM por HTTPS con rutas estables y cabeceras apropiadas.
- Configurar CSP en el portfolio tras pruebas con las exigencias concretas de WASM/Bevy; no debilitarla indiscriminadamente.
- Considerar SRI o manifiestos de integridad cuando la distribución externa lo requiera.
- No incrustar secretos, claves privadas ni credenciales en JS, WASM o assets.
- Aplicar límites estrictos a estados importados y códigos de desafíos.
- Los errores públicos no exponen rutas locales, secretos, stacks completos ni información del host.

### 23.5. Seguridad del usuario

Sin compras, apuestas, publicidad conductual ni competición por dinero. Incluir controles visibles de silencio, pausa y reducción de movimiento. Mantener animaciones discretas y evitar efectos de parpadeo intensos.

### 23.6. Operación sin backend

Los únicos componentes de producción inicialmente necesarios son archivos estáticos y el propio hosting del portfolio. No hay base de datos que respaldar, migraciones de esquemas de servidor ni credenciales de sesión. Sí habrá que mantener builds, versiones de assets, dependencias, tests de navegadores y enlaces de documentación.

## 24. Pruebas y aseguramiento de calidad

### 24.1. Estrategia por capas

| Tipo | Qué comprueba | Entorno |
|---|---|---|
| Unitarias | Validación de tiros, estados, turnos y errores | Rust nativo |
| Reglas | Tabla de verdad de WPA/casual por `ShotReport` | Rust nativo |
| Física | Choques, reposo, bandas, spin, troneras | Headless o escena mínima |
| Propiedades | Invariantes bajo estados/inputs generados | Rust, `proptest` u otro |
| Integración | Coordinación motor–reglas–eventos | Rust/Bevy testbed |
| Web/WASM | Montaje, assets, JS bindings, estados de error | Navegador |
| End to End | Flujo real de Free Play y 8-Ball | Navegador controlado |
| Accesibilidad | Focus, teclado, nombres, mensajes | Automatizada + manual |
| Rendimiento | FPS, carga, GPU, memoria y CPU | Equipos reales definidos |
| Regresión visual | Mesa, bolas, UI en tamaños soportados | Capturas controladas |
| Licencias/seguridad | Dependencias y recursos | CI/revisión humana |

### 24.2. Casos de pruebas físicas obligatorios

1. Tiro recto contra bola estacionaria: sin penetración ni teletransporte.
2. Colisión oblicua con conservación aproximada dentro de tolerancias.
3. Rebote de banda sin bola atravesando collider.
4. Bola con spin lateral y diferencia medible frente a bola sin spin.
5. Follow/draw con respuesta razonable tras el contacto.
6. Golpe potente (saque) sin túnel de colisiones aparente.
7. Bola que roza boca de tronera y vuelve al paño: no cuenta como embolsada.
8. Bola que cae correctamente: embolsado registrado una sola vez.
9. Todas las bolas detenidas: estado transita de `simulating` a `resolving`.
10. Distintas tasas de render: mismo tiro, resultado físico cercano dentro de umbrales.
11. Pausa y reanudación: sin impulsos extras ni salto de tiempo.
12. Posiciones inválidas/`NaN`: rechazo controlado.
13. Restaurar snapshot: sin colisionadores duplicados.
14. Repetir 1.000 tiros automáticos: sin estados imposibles, crash ni fuga creciente evidente.

### 24.3. Casos de reglas obligatorios

- Rack con ocho en centro y grupos en esquinas inferiores.
- Saque legal sin bola embolsada con al menos cuatro bolas alcanzando banda.
- Saque ilegal cuando no se embolsa ninguna bola ni se alcanzan las bandas exigidas.
- Ocho embolsada en saque legal; opciones reglamentarias.
- Ocho embolsada en saque con falta; opciones reglamentarias distintas.
- Mesa abierta después de un saque donde entraron bolas.
- Asignación de grupos solo tras bola llamada y embolsada legalmente.
- Bola equivocada golpeada primero con grupos asignados.
- Scratch ordinario y bola en mano.
- Contacto correcto sin bola embolsada ni banda después del contacto.
- Tiro declarado con bola válida en tronera equivocada.
- Safety declarado y transición de turno.
- Ocho prematura y pérdida de rack.
- Ocho legal con grupo completo y victoria.
- Bola blanca reubicada dentro de banda o superpuesta: rechazo.
- No iniciar otro tiro mientras las bolas giran o se mueven.
- Estado de estancamiento y reglamento de repetición.
- Dos contactos en el mismo paso físico: resolución conforme a política documentada.

### 24.4. Pruebas de ciclo de vida web

Montar → iniciar → pausar → reanudar → resetear → destruir → montar nuevamente. Repetir el ciclo varias veces, inspeccionar listeners, canvases, timers, audio y uso de memoria. Probar error durante carga interrumpida, navegación antes de estar listo y destrucción invocada dos veces. Verificar que otras funciones del portfolio siguen operables tras salir.

### 24.5. Pruebas de UX

Usuarios que no conocen el proyecto deben poder realizar un tiro sin explicación verbal externa. Observar errores de dirección, tiros accidentales, confusión con la fuerza, localización de pausa y dificultad para cambiar de modo. Evaluar pantallas estrechas y controles de teclado por separado.

### 24.6. Criterio de calidad en CI

No fusionar cambios si fallan formato, lint, tests de dominio críticos, construcción del target elegido o auditoría bloqueante. Los benchmarks detectan regresiones pero requieren umbrales basados en hardware reproducible; no comparar FPS de runners de CI heterogéneos como si fueran equivalentes.

## 25. Documentación, licencia y comunidad open source

### 25.1. Documentos a publicar

- `README.md`: qué es CUE, captura real o demo, instalación, ejemplo simple y estado del proyecto.
- `getting-started.md`: requisitos y comandos validados paso a paso.
- `architecture.md`: fronteras entre core, física, juego y bridge.
- `physics-model.md`: parámetros, limitaciones y metodología de calibración.
- `rules-eight-ball.md`: versión oficial, diferencias y comportamiento de casos límite.
- `web-integration.md`: mounting, assets, desmontaje, eventos, errores y frameworks.
- `api.md`: opciones, métodos, tipos, compatibilidad y ejemplos ejecutables.
- `controls.md`: ratón, táctil, teclado y ayudas.
- `accessibility.md`: capacidades, límites y buenas prácticas.
- `browser-support.md`: versiones probadas y limitaciones conocidas.
- `CONTRIBUTING.md`: flujo PR, naming, pruebas y revisión de recursos.
- `SECURITY.md`: cómo reportar vulnerabilidades de manera responsable.
- `CHANGELOG.md`: cambios visibles, migraciones y versiones.

### 25.2. Licencia

**Propuesta:** MIT para el código original, después de verificar su autoría y compatibilidad. No significa automáticamente que los modelos, audios, texturas o tipografías tengan esa licencia. Definir explícitamente la política de assets (por ejemplo, originales bajo licencia elegida o recursos de terceros con aviso independiente) y publicar los archivos `LICENSE`, `NOTICE` y atribuciones pertinentes.

No es asesoría jurídica; si existen colaboradores externos o assets comerciales, confirmar derechos antes de redistribuir.

### 25.3. Contribuciones

Organizar issues con etiquetas como `bug`, `physics`, `rendering`, `accessibility`, `documentation`, `good first issue` y `breaking change`. Exigir un caso de reproducción, pruebas cuando corresponda y una explicación de cualquier cambio en el comportamiento físico. Aceptar PRs pequeños y trazables.

### 25.4. Política de comunicación honesta

No describir CUE como «WPA compliant», «realistic physics» o «works everywhere» sin evidencia adecuada. En README, identificar qué es experimental, qué está implementado y qué falta. Los vídeos promocionales no sustituyen pruebas reproducibles.

## 26. CI/CD, distribución y versiones

### 26.1. Pipeline de pull request

1. Validar formato con `cargo fmt`.
2. Analizar Rust con `cargo clippy`.
3. Compilar y ejecutar tests del workspace.
4. Compilar versión WASM configurada (con caché controlada).
5. Ejecutar tests de integración web disponibles.
6. Comprobar licencias/dependencias y listado de recursos.
7. Reportar tamaño de los artefactos y regresiones notables.
8. Bloquear merge ante error crítico.

### 26.2. Pipeline de versión

- Crear tag semántico aprobado (`v1.0.0`, etc.).
- Compilar con toolchain y lockfile fijados.
- Generar WASM, JS glue, assets y documentación.
- Crear checksums, release notes y changelog.
- Probar paquete dentro de un proyecto consumidor limpio.
- Publicar release en GitHub.
- Publicar paquete npm cuando la API esté estabilizada y el empaquetado probado.
- Actualizar demo/portfolio intencionalmente, con rollback disponible.

### 26.3. Política de versionado

Aplicar SemVer a la API externa. Cambios incompatibles en métodos, eventos, opciones, significado de estados o formato de snapshots requieren cambio de versión correspondiente. Un cambio de físicas que modifica resultados puede ser un cambio observable aunque no rompa la firma de métodos; se documentará mediante versión del `physics_profile` y changelog.

### 26.4. Publicaciones y canales

- `alpha`: prototipos sin garantías, API mutable.
- `beta`: funcionalidades previstas completas, fase intensiva de pruebas.
- `1.0.0`: API y juego base estables según definición de terminado.

No llamar `1.0` a un ejecutable que solo se muestra dentro del portfolio sin instrucciones reproducibles para terceros.

### 26.5. Reversibilidad

Guardar artefactos versionados y permitir regresar a una versión de CUE que ya funcionaba en el portfolio. Cambiar de versión debe ser independiente de alterar páginas no relacionadas.

---
## 27. Roadmap de ejecución

El roadmap se organiza por **evidencias de salida**, no por fechas arbitrarias. La duración depende de experiencia en Rust, disponibilidad de tiempo, investigación física, calidad de los assets y compatibilidad web.

### Fase 0 — Definición y preparación

**Objetivo:** convertir este documento en decisiones ejecutables.

**Tareas:**

1. Crear repositorio Git independiente y ajustar README inicial.
2. Elegir nombre de organización/namespace público disponible.
3. Bloquear toolchain y dependencias compatibles.
4. Configurar Rust, formato, lint, pruebas y CI mínima.
5. Seleccionar el perfil de mesa y unidades físicas.
6. Redactar primeros ADR: Rust/Bevy, Rapier 3D, WASM/WebGL2, sin backend.
7. Definir el dispositivo de prueba desktop y móvil.
8. Inventariar assets necesarios y estrategia de creación.

**Entregable:** esqueleto compilable del workspace, CI verde, decisiones registradas.  
**Criterio de salida:** `cargo check` y `cargo test` de base funcionan en una instalación limpia.

### Fase 1 — Viabilidad técnica (*vertical slice*)

**Objetivo:** probar lo más riesgoso antes de pulir.

**Tareas:**

1. Crear una escena Bevy 3D con mesa básica.
2. Generar 3 esferas físicas y colliders de banda.
3. Aplicar tiro mediante comando de input.
4. Separar scheduling de físicas del render.
5. Detectar primer contacto, rebote y reposo.
6. Crear build WASM y ejecutar dentro de un contenedor HTML.
7. Probar ratón y touch en navegador real.
8. Medir FPS, tiempo hasta interacción y peso de la build.
9. Intentar desmontar y volver a montar sin errores críticos.
10. Registrar defectos de spin, sombras, audio y compatibilidad.

**Entregable:** prototipo jugable en navegador con pocos objetos y métricas.  
**Criterio de salida:** existe prueba de stack funcional en desktop y al menos un dispositivo móvil objetivo. Si no, evaluar reducción del alcance gráfico o cambio del adaptador físico antes de continuar.

### Fase 2 — Motor de billar

**Objetivo:** crear un modelo suficientemente estable para sostener reglas.

**Tareas:**

1. Definir contrato de órdenes, estado y eventos.
2. Completar mesa con proporciones y seis troneras.
3. Añadir 16 bolas identificadas y rack válido.
4. Implementar contactos bola-bola y bola-banda robustos.
5. Calibrar deslizamiento, rodadura, restitución y frenado.
6. Añadir y probar spin en los ejes soportados.
7. Implementar embolsado con detección y confirmación.
8. Construir `ShotReport` e invariantes físicas.
9. Añadir escenarios automatizados y perfiles de calibración.
10. Medir rendimiento tras sucesivos tiros.

**Entregable:** simulación con tiros fiables y estado autoritativo.  
**Criterio de salida:** pasan los casos críticos físicos y no se detectan colisiones perdidas bloqueantes en el escenario de prueba.

### Fase 3 — Jugabilidad Free Play

**Objetivo:** lograr la experiencia que verá primero un visitante.

**Tareas:**

1. Apuntado con puntero, fuerza y golpe.
2. Interacción táctil y esquema de teclado básico.
3. Tutorial breve, ayudas, selección de tema y pausa.
4. Reinicio y reposición segura de blanca.
5. Marcadores/indicadores visuales y HUD discreto.
6. Audio y ajustes locales opcionales.
7. Corrección de scroll, foco y resize.
8. Estados de carga/error y regreso al portfolio.
9. Pruebas reales de uso con personas nuevas.

**Entregable:** Free Play jugable dentro de `/cue`.  
**Criterio de salida:** un visitante puede hacer un tiro, comprender su resultado, reiniciar y salir sin explicaciones externas.

### Fase 4 — 8-Ball local y motor de reglas

**Objetivo:** incorporar una partida reglada, automatizable y comprobable.

**Tareas:**

1. Implementar `Ruleset` y evaluador de `ShotReport`.
2. Estados de rack, break, mesa abierta, grupos y turno.
3. Llamada de bola/tronera y safety.
4. Faltas, bola en mano y decisiones de saque.
5. Continuidad de turno, victoria, derrota y estancamiento.
6. UI para dos jugadores en mismo dispositivo.
7. Matriz automatizada de casos WPA y excepciones.
8. Definir cuándo se llama oficialmente `eight-ball-wpa` y cuándo permanece `casual`.

**Entregable:** partida local completa con el perfil de reglas declarado.  
**Criterio de salida:** todas las transiciones normativas incluidas tienen pruebas; no se anuncia cumplimiento oficial con excepciones sin implementar.

### Fase 5 — Dirección visual, audio y pulido

**Objetivo:** convertir un motor funcional en una experiencia distintiva.

**Tareas:**

1. Modelado final de mesa, bolas y taco.
2. Materiales y texturas optimizadas.
3. Cámara definitiva y correcciones de proyección.
4. Animación de golpe sincronizada con físicas.
5. Sonidos calibrados a contactos.
6. Temas principales y modo de alto contraste.
7. Calidad gráfica adaptativa.
8. Optimización de assets, WASM, frame time y memoria.
9. Corrección de defectos visuales y de interacción móvil.

**Entregable:** experiencia lista para demo pública.  
**Criterio de salida:** prueba visual y jugable aprobada en la matriz de dispositivos, sin regresiones físicas.

### Fase 6 — Biblioteca open source y publicación 1.0

**Objetivo:** que un tercero pueda utilizar CUE fuera del portfolio.

**Tareas:**

1. Estabilizar bridge JS y documentación de API.
2. Empaquetar WASM, ESM y assets.
3. Crear ejemplo sin framework externo al portfolio.
4. Verificar montaje, destrucción y reinstanciación.
5. Preparar licencia, avisos de assets y política de seguridad.
6. Completar guía de controles y compatibilidad.
7. Probar npm y la instalación de un consumidor limpio.
8. Establecer release/tag/changelog.
9. Actualizar portfolio para consumir una versión publicada.
10. Publicar código, documentación y versión verificable.

**Entregable:** release funcional, repo público y demo embebida.  
**Criterio de salida:** una persona externa puede integrar y ejecutar CUE siguiendo solo la documentación pública.

### Fase 7 — Evolución posterior

Investigar desafíos reproducibles, exportación/importación de escenarios, replays, modos nuevos, IA local, mayor precisión de efecto, builders, adaptadores adicionales y posible física dedicada. Cada ampliación requiere una justificación de producto y pruebas de impacto en rendimiento/complexidad.

## 28. Backlog funcional detallado

### 28.1. Prioridad P0 — indispensable para prototipo/MVP

| ID | Trabajo | Aceptación verificable |
|---|---|---|
| CUE-001 | Workspace Rust y CI base | Compilación y tests reproducibles |
| CUE-002 | Escena 3D básica Bevy | Mesa y cámara visibles |
| CUE-003 | Integración Rapier 3D | Esferas dinámicas y colliders estáticos |
| CUE-004 | Paso fijo de simulación | Resultado independiente del render dentro de tolerancias |
| CUE-005 | Golpe de blanca | Input validado produce un impulso una sola vez |
| CUE-006 | Bandas y suelo | No penetraciones/túneles frecuentes en casos críticos |
| CUE-007 | Seis troneras | Embolsado confirmado y event registrado |
| CUE-008 | Rack 8-ball | 16 bolas con IDs y posiciones válidas |
| CUE-009 | Detección de reposo | Se cierra el tiro en condiciones estables |
| CUE-010 | Apuntado desktop | No genera disparos accidentales en pruebas |
| CUE-011 | Apuntado móvil | Se puede disparar con gestos predecibles |
| CUE-012 | Free Play | Ciclo disparar/reiniciar continuo |
| CUE-013 | Build WASM | Se ejecuta desde HTTP en navegador de referencia |
| CUE-014 | Montaje en portfolio | No requiere abrir aplicación externa |
| CUE-015 | Pausa/destrucción | Sin bucles o listeners persistentes detectables |
| CUE-016 | Estado de carga/error | No aparece pantalla en blanco sin explicación |

### 28.2. Prioridad P1 — indispensable para 1.0 pública

| ID | Trabajo | Aceptación verificable |
|---|---|---|
| CUE-017 | Rulesets | Evaluación sin Bevy/render |
| CUE-018 | Break y mesa abierta | Coincide con casos de reglas seleccionadas |
| CUE-019 | Asignación de grupos | Estado correcto tras tiro llamado legal |
| CUE-020 | Bola/tronera declarada | Puede introducirse desde desktop y móvil |
| CUE-021 | Faltas y bola en mano | Se procesa correctamente cada caso soportado |
| CUE-022 | Turnos/resultado | No hay cambios dobles ni victorias espurias |
| CUE-023 | Modo 8-Ball local | Dos personas terminan una partida completa |
| CUE-024 | Accesibilidad base | Teclado, foco, contraste y texto de estado |
| CUE-025 | Sistema de audio | Respeta gesto usuario y mute |
| CUE-026 | Temas/calidad | Cambios gráficos no cambian física |
| CUE-027 | API web estable | Montaje/inicio/eventos/destrucción probados |
| CUE-028 | Demo externa | Funciona sin el repositorio del portfolio |
| CUE-029 | Documentación completa | Instalación reproducible por tercero |
| CUE-030 | Seguridad/licencias | Revisión y atribución completadas |
| CUE-031 | CI Web/WASM | Build y pruebas críticas automatizadas |
| CUE-032 | Release 1.0 | Publicada con changelog/versiones coherentes |

### 28.3. Prioridad P2 — ampliaciones opcionales

| ID | Trabajo | Dependencia |
|---|---|---|
| CUE-033 | Challenges | Motor estable y formato de escena seguro |
| CUE-034 | Códigos/URL compartibles | Serialización versionada + validación |
| CUE-035 | Replays | Tiros determinables + snapshot completo |
| CUE-036 | Editor de configuraciones | API de estados segura |
| CUE-037 | IA local de práctica | Evaluación/heurísticas de tiros |
| CUE-038 | Adaptador React | API JS estabilizada |
| CUE-039 | Adaptadores Vue/Svelte | Demanda real de terceros |
| CUE-040 | Perfil WebGPU | Compatibilidad y beneficio demostrado |
| CUE-041 | Núcleo físico alternativo | Pruebas que justifiquen sustitución |
| CUE-042 | Builds nativas | Mantenimiento y distribución definidos |

## 29. Riesgos y medidas de mitigación

| Riesgo | Efecto | Mitigación concreta |
|---|---|---|
| Bevy WASM demasiado pesado | Carga lenta en portfolio | Vertical slice temprano; medir; reducir features, assets y calidad |
| WebGL2/WebGPU desigual | Usuarios sin juego | Backend objetivo probado, matriz por navegador, fallback informativo |
| Plugin Bevy/Rapier incompatible tras update | Build roto | Lockfile y pruebas de actualización separadas |
| Rapier no reproduce bien spin/paño | Golpes poco naturales | Calibración, escenarios de referencia, adaptador sustituible |
| Input móvil impreciso | Abandono de jugadores | Controles independientes del ratón, tests con personas reales |
| Reglas WPA incompletas | Resultados incorrectos | Ruleset separado, pruebas y etiqueta `casual` hasta cumplimiento |
| API acoplada al portfolio | No reutilizable | Demo Vanilla independiente y contrato público desde temprano |
| Rendimiento móvil bajo | Tirones/temperatura | Calidad adaptativa, límites de resolución, perfiles de GPU |
| Ciclo de vida defectuoso | Fugas de memoria al navegar | Tests repetidos de mount/destroy y medición de recursos |
| Assets ajenos sin licencia | Incompatibilidad open source | Inventario y política de atribuciones desde Fase 0 |
| Exceso de funciones | Proyecto interminable | Fases con exit criteria, congelación de alcance 1.0 |
| Curva de aprendizaje Rust/Bevy | Demoras técnicas | Prototipos pequeños, módulos sencillos y documentación |
| Determinismo entre plataformas | Replays inconsistentes | Paso fijo, semillas, versión de perfil y tolerancias explícitas |
| Accesibilidad limitada por canvas | Barreras de uso | Capa HTML accesible y control alternativo de teclado |
| Bloqueo de audio/autoplay | Silencio inesperado | Inicialización tras gesto de usuario, feedback visual |
| Falsos embolsados o faltas | Frustración | Collider de tronera calibrado y reportes verificables |
| Dependencias vulnerables | Riesgo en terceros | Auditoría, fijación de versiones y política de actualización |

### 29.1. Riesgos que pueden invalidar el stack

Si el vertical slice demuestra que Bevy 3D/WASM no alcanza un arranque o rendimiento aceptables para el portfolio, la decisión correcta **no es mantener el stack a cualquier coste**. Se debe documentar el hallazgo y considerar: simplificar representación 3D, reducir features, separar runtime y renderer o adoptar un render más ligero conservando Rust como core. CUE es un producto; el stack debe servir al producto.

## 30. Criterios de aceptación y definición de terminado

### 30.1. «Terminado» para la experiencia MVP

- [ ] CUE se abre dentro del portfolio sin login ni redirección externa.
- [ ] El juego carga mediante HTTPS en la matriz de navegadores declarada.
- [ ] Hay mesa, seis troneras, taco y conjunto completo de bolas.
- [ ] El usuario puede apuntar, regular potencia y golpear.
- [ ] Los movimientos no muestran penetraciones o saltos graves en escenarios críticos.
- [ ] Las bolas giran visualmente de acuerdo con el estado físico.
- [ ] El embolsado se contabiliza exactamente una vez por bola.
- [ ] El modo Free Play se puede reiniciar.
- [ ] Se puede pausar, silenciar y salir de la experiencia.
- [ ] Existen controles adaptados a ratón y táctil; el teclado cubre funciones principales.
- [ ] El canvas se ajusta al contenedor y el cambio de tamaño no rompe el input.
- [ ] Los fallos de GPU/WASM se comunican con salida segura.
- [ ] Montar y destruir repetidamente no deja bucles activos ni rompe el portfolio.
- [ ] Los casos automatizados de motor considerados críticos pasan.

### 30.2. «Terminado» para 8-Ball local

- [ ] Dos jugadores pueden alternar turnos en un dispositivo.
- [ ] Saque y rack respetan el perfil elegido.
- [ ] La mesa abierta y los grupos se asignan correctamente.
- [ ] Se puede declarar bola y tronera fuera del saque.
- [ ] Se pueden declarar tiros de seguridad cuando lo requiera el perfil.
- [ ] Se detectan y penalizan faltas automatizables declaradas.
- [ ] La blanca en mano se coloca sin posiciones imposibles.
- [ ] La victoria y derrota relacionadas con la ocho son correctas.
- [ ] Los casos especiales del saque no se tratan como reglas ordinarias.
- [ ] No se afirma equivalencia WPA si hay excepciones pendientes.
- [ ] Las reglas se pueden probar sin inicializar Bevy ni el navegador.

### 30.3. «Terminado» para librería open source 1.0

- [ ] Existe repo separado, público, con licencia válida.
- [ ] El código principal es Rust; las dependencias del frontend están documentadas.
- [ ] La API web se instala e importa en un proyecto externo.
- [ ] Un ejemplo Vanilla independiente muestra cómo montar y desmontar.
- [ ] Se publican archivos estáticos/versionados y guía de hosting.
- [ ] Existe referencia de opciones, métodos, eventos y errores.
- [ ] Está documentada la matriz real de compatibilidad.
- [ ] Tests, lint, build WASM y auditoría esencial corren en CI.
- [ ] No se redistribuyen assets sin autorización/licencia compatible.
- [ ] La demo del portfolio consume la misma distribución pública.
- [ ] Hay changelog, política de contribuciones y canal de reportes de seguridad.
- [ ] Un desarrollador externo ha conseguido integrarlo siguiendo la guía, sin asistencia privada.

### 30.4. Definición general de terminado para un ticket

Un trabajo se considera terminado cuando su comportamiento está especificado, implementado, probado, revisado y documentado; no rompe las fronteras de arquitectura; respeta accesibilidad y licencias; no produce regresiones graves de rendimiento; y se integra sin aumentar inadvertidamente el alcance del proyecto.

## 31. Decisiones abiertas y registro de arquitectura

El propósito de esta sección es **distinguir ambigüedades reales de decisiones ya tomadas**. Las respuestas se registrarán en ADR, preferentemente antes de la fase que las necesite.

| ID | Decisión pendiente | Alternativas | Cuándo resolver |
|---|---|---|---|
| ADR-005 | Aspecto principal de la cámara | Cenital pura / oblicua leve | Fase 1–3 |
| ADR-006 | Apariencia base de la mesa | Classic / studio / nueva dirección | Fase 3–5 |
| ADR-007 | Paso físico final | 120 Hz / 60 Hz + subpasos | Fase 1–2 |
| ADR-008 | Modelo de fricción/efecto | Rapier calibrado / módulo especializado | Fase 2 |
| ADR-009 | Primer modo al entrar | Free Play inmediato / selector | Fase 3 |
| ADR-010 | Mecánica precisa de potencia | Arrastre separado / pull-release híbrido | Fase 3 |
| ADR-011 | UI de llamada bola-tronera | Overlay visual / selectores accesibles | Fase 4 |
| ADR-012 | Backend gráfico definitivo web | WebGL2 único / builds diferenciadas | Fase 1/5 |
| ADR-013 | Persistencia local | Solo preferencias / incluir estado parcial | Fase 3 |
| ADR-014 | Configuración de la URL del desafío | JSON comprimido + validado / ID derivado | Fase 7 |
| ADR-015 | Namespace de distribución | `@tuletta/cue` u otro disponible | Fase 0/6 |
| ADR-016 | Licencia exacta de assets | Licencia abierta propia / combinación con atribuciones | Fase 0/6 |
| ADR-017 | 8-Ball inicial | Casual primero / WPA completo antes de preview | Fase 3–4 |
| ADR-018 | Multinstancia web | No soportada v1 / experimental | Fase 6 |
| ADR-019 | Idiomas iniciales de UI | Inglés / español+inglés | Fase 3 |

### 31.1. Plantilla ADR recomendada

```markdown
# ADR-XXXX — Nombre

- Estado: propuesto | aceptado | sustituido
- Fecha:
- Contexto:
- Problema que resolvemos:
- Opciones consideradas:
- Decisión:
- Consecuencias positivas:
- Costes y riesgos:
- Evidencia (pruebas/benchmarks):
- Condiciones de revisión:
```

### 31.2. Lo que se debe validar primero

1. **Que Rust + Bevy + Rapier 3D en WASM ejecuta una mesa simple en los navegadores objetivo.**
2. **Que el tamaño/carga no deteriora el portfolio.**
3. **Que la física de spin y bandas puede alcanzar la sensación deseada sin programar todo desde cero.**
4. **Que el control móvil resulta agradable.**
5. **Que se puede montar/destruir de forma limpia dentro de un sitio anfitrión.**

Si esos cinco puntos se validan, el riesgo principal del proyecto disminuye notablemente.

## 32. Glosario

| Término | Significado en CUE |
|---|---|
| **API** | Contrato público con el que otras aplicaciones controlan CUE |
| **Asset** | Modelo, textura, audio, shader u otro recurso de presentación |
| **Bevy** | Motor de juego en Rust que gestiona ECS y representación |
| **Break** | Tiro inicial de apertura del triángulo |
| **CCD** | *Continuous Collision Detection*; técnica para evitar ciertos choques omitidos por objetos rápidos |
| **Collider** | Geometría usada en detección/resolución de colisiones |
| **Cue ball / blanca** | Bola que golpea el taco |
| **Cue stick / taco** | Elemento controlado para aplicar el tiro |
| **Draw** | Efecto de retroceso de la blanca debido a rotación inferior |
| **ECS** | *Entity–Component–System*; organización de lógica/entidades utilizada por Bevy |
| **Free Play** | Modo de práctica sin competición reglamentaria |
| **Glue JS** | Código JavaScript generado/escrito para conectar WASM con el navegador |
| **Head string** | Línea reglamentaria que delimita el área de saque |
| **Host** | Web o aplicación que integra el juego |
| **HUD** | Información superpuesta y controles de contexto |
| **MVP** | Primera versión usable que demuestra el valor principal |
| **Pocket / tronera** | Abertura donde puede entrar una bola |
| **Rapier** | Motor de físicas rígidas en Rust |
| **Rack** | Disposición reglamentaria inicial de las bolas |
| **Restitution** | Medida del rebote en contacto |
| **Ruleset** | Perfil versionado con las reglas aplicables a una partida |
| **Safety** | Tiro anunciado en el que se cede la mesa conforme al reglamento |
| **ShotReport** | Resumen de contactos y sucesos necesarios para decidir un tiro |
| **Spin** | Rotación de la bola que afecta trayectoria y contactos |
| **WASM** | WebAssembly, formato ejecutable por navegadores compatibles |
| **WebGL2** | API de gráficos para navegador basada en OpenGL ES 3.0 |
| **WebGPU** | API gráfica moderna de navegador; soporte condicionado a navegador y backend |
| **WPA** | World Pool-Billiards Association, referencia normativa seleccionada |

## 33. Fuentes técnicas y normativas

Fuentes consultadas para la versión de este documento. Son referencias **externas**, no bibliotecas cuyos contenidos se deban copiar en el repositorio. Las versiones, fechas y políticas pueden cambiar: revisar cuando se inicie una fase o se actualicen dependencias.

### 33.1. Lenguaje y motor

- **Rust oficial:** <https://www.rust-lang.org/>
- **Cargo Book:** <https://doc.rust-lang.org/cargo/>
- **Bevy, anuncio 0.20 (2026-10-08):** <https://bevy.org/news/bevy-0-20/>
- **Bevy, repositorio:** <https://github.com/bevyengine/bevy>
- **Bevy, ejemplos web/WASM:** <https://github.com/bevyengine/bevy/blob/main/examples/README.md>
- **Bevy, opciones de render WebGL2/WebGPU:** <https://github.com/bevyengine/bevy/blob/main/Cargo.toml>

### 33.2. Física

- **Rapier, portal oficial:** <https://rapier.rs/docs/>
- **Rapier, plugin Bevy (primeros pasos):** <https://rapier.rs/docs/user_guides/bevy_plugin/getting_started_bevy/>
- **Rapier, configuración de paso fijo:** <https://rapier.rs/docs/user_guides/bevy_plugin/integration_parameters/>
- **Rapier, determinismo y condiciones:** <https://rapier.rs/docs/user_guides/bevy_plugin/determinism/>
- **Rapier, contacto y fricción:** <https://rapier.rs/docs/user_guides/rust/colliders/>
- **`bevy_rapier3d` 0.37.0 y dependencias:** <https://docs.rs/crate/bevy_rapier3d/0.37.0>

### 33.3. WebAssembly e integración

- **Guía de `wasm-bindgen`:** <https://wasm-bindgen.github.io/wasm-bindgen/>
- **`wasm-bindgen`, despliegue:** <https://wasm-bindgen.github.io/wasm-bindgen/reference/deployment.html>
- **MDN, compilación de Rust a WASM:** <https://developer.mozilla.org/en-US/docs/WebAssembly/Guides/Rust_to_Wasm>
- **`wasm-pack`, herramientas:** <https://github.com/wasm-bindgen/wasm-pack>
- **MDN, autoplay y Web Audio:** <https://developer.mozilla.org/en-US/docs/Web/Media/Guides/Autoplay>

### 33.4. Reglas y accesibilidad

- **WPA, página oficial de reglas:** <https://wpapool.com/rules/>
- **WPA, *Rules of Play*, efectivo 2025-09-15, apartado 4 (8-Ball):** <https://www.wpapool.com/wp-content/uploads/2026/01/2026.01.02-WPA-Rules.pdf>
- **W3C, WCAG 2.2:** <https://www.w3.org/TR/WCAG22/>
- **W3C, técnica de control mediante teclado:** <https://www.w3.org/WAI/WCAG22/Techniques/general/G202>

---

## Declaración final de alcance

**CUE no es simplemente un juego insertado en un portfolio.** Es un proyecto autónomo, reutilizable y abierto, cuya primera demostración pública es un billar integrado en dicho portfolio. Su arquitectura principal es Rust, la escena se construye con Bevy, la física inicial utiliza Rapier 3D y la ejecución web se resuelve con WebAssembly. No requiere backend ni usuarios para funcionar.

La calidad perseguida es doble: **una experiencia de billar convincente para cualquier visitante y un código comprensible, probado y aprovechable para otros desarrolladores**. La primera obligación de ingeniería será demostrar que el stack funciona realmente en navegador y que se integra limpiamente; después vendrán la precisión, los modos y el pulido visual.

**Próxima acción recomendada:** ejecutar las Fases 0 y 1 para producir una mesa de prueba en WebAssembly con una blanca, dos bolas objetivo, bandas, un tiro controlado, un contenedor HTML y mediciones reales. Solo tras superar esa validación se consolidará el diseño detallado del producto.

> **Control documental:** actualizar esta especificación cuando una prueba contradiga una hipótesis, se apruebe un ADR o cambie la definición de MVP/1.0. Cada nueva versión del documento debe conservar fecha, cambios y razones; no corregir silenciosamente decisiones arquitectónicas importantes.
