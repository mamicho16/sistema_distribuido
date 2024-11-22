# Simulación de Sistema Distribuido en Rust

## Descripción del Proyecto

Este proyecto es una simulación de un sistema distribuido implementado en Rust, diseñado para modelar el comportamiento de nodos (computadoras) en una red que necesitan coordinar tareas, gestionar recursos compartidos y manejar fallos de manera eficiente. La simulación abarca aspectos clave de los sistemas distribuidos, incluidos:

- **Programación Asíncrona**: Aprovecha las capacidades asíncronas de Rust y el runtime Tokio para operaciones no bloqueantes.
- **Gestión de Recursos**: Simula la asignación y liberación de recursos computacionales (RAM, espacio en disco, hilos) entre procesos.
- **Exclusión Mutua**: Implementa el algoritmo de Ricart-Agrawala para el acceso sincronizado a recursos compartidos.
- **Detección y Recuperación de Fallos**: Maneja fallos de nodos y procesos con estrategias adecuadas para mantener la integridad del sistema.
- **Mecanismo de Consenso**: Utiliza un protocolo de votación entre nodos para acordar acciones, imitando algoritmos de consenso en sistemas distribuidos.

## Solución Propuesta

La simulación busca proporcionar un modelo integral de un sistema distribuido mediante:

- **Creación de una Arquitectura Escalable de Nodos**: Cada nodo opera de manera independiente, procesando tareas y comunicándose con otros nodos según sea necesario.
- **Implementación de Comunicación Asíncrona**: Los nodos interactúan mediante el envío de mensajes asíncronos, asegurando operaciones no bloqueantes y una utilización eficiente de recursos.
- **Gestión Eficiente de Recursos**: Una estructura centralizada (`Session`) gestiona los recursos disponibles y los nodos los solicitan antes de ejecutar procesos.
- **Asegurar Consistencia y Sincronización de Datos**: Con el algoritmo de Ricart-Agrawala, los nodos logran exclusión mutua al acceder a recursos compartidos.
- **Manejo Robusto de Fallos**: El sistema detecta fallos tanto a nivel de procesos como de nodos, e inicia protocolos de recuperación para mantener la estabilidad del sistema.
- **Facilitar Consenso**: Los nodos participan en un mecanismo de votación para acordar acciones críticas, asegurando decisiones colectivas.

## Diseño de la Arquitectura

### Componentes

La simulación se estructura en varios componentes clave:

1. **Nodo** (`node.rs`):
   - Representa un nodo individual en el sistema distribuido.
   - Gestiona procesos activos y maneja la comunicación con otros nodos.
   - Ejecuta procesos de forma asíncrona.

2. **Proceso** (`process.rs`):
   - Define las tareas a ejecutar por los nodos.
   - Especifica los recursos necesarios para la ejecución.

3. **Sesión** (`session.rs`):
   - Actúa como orquestador de la simulación.
   - Gestiona la colección de nodos y la asignación de recursos.
   - Maneja la exclusión mutua y los mecanismos de votación.

4. **Recursos** (`resource.rs`):
   - Representa los recursos computacionales disponibles en el sistema (RAM, espacio en disco, hilos).
   - Proporciona métodos para la asignación y liberación.

5. **Protocolos de Mensajes** (`message.rs`):
   - Define las estructuras para la comunicación entre nodos, incluyendo acciones, votos y solicitudes.

## Flujo de Datos

1. **Inicialización**:
   - Los nodos y procesos se crean y registran en la `Session`.
   - Se definen los recursos totales del sistema.

2. **Asignación de Procesos**:
   - Los procesos se asignan a nodos según la disponibilidad de recursos y el balance de carga.

3. **Solicitud y Asignación de Recursos**:
   - Los nodos solicitan recursos a la `Session` antes de ejecutar procesos.
   - La `Session` asigna recursos si están disponibles, asegurando que no haya sobreasignación.

4. **Ejecución de Procesos**:
   - Los nodos ejecutan procesos de forma asíncrona, simulando trabajo con retrasos no bloqueantes.

5. **Exclusión Mutua**:
   - Los nodos coordinan el acceso a recursos compartidos utilizando el algoritmo de Ricart-Agrawala.
   - Las solicitudes y respuestas se gestionan para garantizar acceso exclusivo.

6. **Manejo de Fallos**:
   - Los nodos detectan y reportan fallos, iniciando un proceso de votación para acciones de recuperación.
   - La `Session` maneja la eliminación de nodos y la redistribución de procesos según sea necesario.

7. **Consenso y Votación**:
   - Los nodos participan en votaciones para alcanzar consenso sobre acciones críticas, como el manejo de fallos.
   - La `Session` ejecuta acciones basadas en los resultados de las votaciones.


## Mecanismos de Comunicación

### Intercambio de Mensajes

La comunicación entre los nodos y la `Session` se facilita mediante estructuras de mensajes definidas en `message.rs`. Los mensajes clave incluyen:

- **Acción**: Representa acciones propuestas, como manejar fallos o redistribuir procesos.
- **Voto**: Los nodos emiten votos (`Approve` o `Reject`) sobre las acciones propuestas.
- **Solicitud**: Usado en el algoritmo de exclusión mutua para solicitar acceso a recursos compartidos.


## Sincronización

### Exclusión Mutua con el Algoritmo Ricart-Agrawala

Para sincronizar el acceso a recursos compartidos, la simulación implementa el algoritmo Ricart-Agrawala:

1. **Fase de Solicitud**:
   - Un nodo envía una solicitud con marca de tiempo a todos los demás nodos.
   - Las solicitudes se colocan en una cola para ser procesadas en orden.

2. **Fase de Respuesta**:
   - Los nodos deciden si conceder acceso inmediato o diferir la respuesta según las marcas de tiempo.
   - Las respuestas se envían al nodo solicitante si puede proceder.

3. **Fase de Ejecución**:
   - Una vez que un nodo recibe respuestas de todos los demás, obtiene acceso al recurso.
   - Tras su uso, el nodo libera el recurso y envía respuestas diferidas.


## Gestión de Fallos

### Detección de Fallos de Procesos

- Los nodos monitorean la ejecución de procesos.
- Si un proceso falla, el nodo:
  - Elimina el proceso de su lista activa.
  - Libera los recursos asociados.
  - Notifica a la `Session` o propone una acción.

### Manejo de Fallos de Nodos

- Los nodos detectan fallos (e.g., incapacidad para proceder) y los reportan a la `Session`.
- La `Session` inicia un proceso de votación para acordar cómo manejar el fallo.

### Mecanismo de Consenso

- El protocolo de votación asegura que todos los nodos participen en decisiones críticas.
- Se requiere un consenso mayoritario para proceder con acciones como la eliminación de nodos o redistribución de procesos.