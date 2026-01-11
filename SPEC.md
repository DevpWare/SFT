# DevpWareSoft v2.0 - Especificación Técnica

> Analizador de dependencias para proyectos Delphi con visualización 3D interactiva.
> Migración a Tauri para distribución como aplicación de escritorio multiplataforma.

---

## Stack Tecnológico

### Frontend

| Tecnología         | Versión | Uso                          |
| ------------------ | ------- | ---------------------------- |
| React              | 19.x    | Framework UI                 |
| TypeScript         | 5.x     | Tipado estático              |
| Vite               | 7.x     | Build tool                   |
| Three.js           | latest  | Renderizado 3D               |
| @react-three/fiber | latest  | React renderer para Three.js |
| @react-three/drei  | latest  | Helpers para R3F             |
| shadcn/ui          | latest  | Componentes UI               |
| Tailwind CSS       | 4.x     | Estilos utilitarios          |
| Framer Motion      | 11.x    | Animaciones                  |
| Zustand            | 5.x     | Estado global                |
| TanStack Query     | 5.x     | Cache y sincronización       |
| Radix UI           | latest  | Primitivos accesibles        |

### Backend (Tauri)

| Tecnología | Uso                            |
| ---------- | ------------------------------ |
| Rust       | Core del backend               |
| Tauri 2.x  | Framework desktop              |
| serde      | Serialización JSON             |
| regex      | Parsing de código              |
| rusqlite   | Base de datos local            |
| tokio      | Async runtime                  |
| walkdir    | Escaneo de directorios         |
| zip        | Manejo de archivos comprimidos |

### Plataformas Soportadas

- Windows 10/11 (x64, ARM64)
- macOS 12+ (Intel, Apple Silicon)
- Linux (Ubuntu 20.04+, Fedora 38+, Arch)

---

## REQUERIMIENTOS FUNCIONALES

### RF-01: Gestión de Proyectos

#### RF-01.1: Crear Proyecto

- Crear nuevo proyecto de análisis
- Asignar nombre y descripción
- Seleccionar carpeta raíz del código fuente
- Configurar lenguaje principal (Delphi por defecto)

#### RF-01.2: Abrir Proyecto Existente

- Listar proyectos recientes
- Buscar proyectos por nombre
- Importar proyecto desde archivo `.devware`
- Validar integridad de datos al abrir

#### RF-01.3: Configuración de Proyecto

- Definir extensiones a incluir/excluir
- Configurar directorios ignorados
- Establecer encoding de archivos
- Guardar configuración por proyecto

---

### RF-02: Escaneo de Código Fuente

#### RF-02.1: Escaneo de Carpetas

- Seleccionar carpeta raíz mediante diálogo nativo
- Escaneo recursivo de subdirectorios
- Filtrado por extensiones configuradas
- Exclusión de carpetas de build/cache
- Barra de progreso durante escaneo
- Cancelación de escaneo en curso

#### RF-02.2: Soporte de Archivos Comprimidos

- Abrir archivos ZIP directamente
- Descompresión a directorio temporal
- Limpieza automática de temporales
- Soporte para RAR y 7z (futuro)

#### RF-02.3: Detección de Pares de Archivos

- Detectar pares .pas ↔ .dfm automáticamente
- Detectar pares .pas ↔ .lfm (Lazarus)
- Agrupar archivos relacionados
- Reportar archivos huérfanos

#### RF-02.4: Indexación de Estructura

- Generar árbol de directorios
- Calcular tamaño de cada archivo
- Detectar archivos duplicados
- Generar hash MD5 para identificación

---

### RF-03: Parsing de Código Delphi

#### RF-03.1: Parsing de Unidades (.pas)

- Extraer declaración de unidad (`unit NombreUnidad;`)
- Extraer sección `interface` y `implementation`
- Detectar dependencias (`uses Unit1, Unit2;`)
- Separar uses de interface vs implementation

#### RF-03.2: Extracción de Tipos

- Extraer clases (`TMyClass = class(TParent)`)
- Extraer records (`TMyRecord = record`)
- Extraer enumeraciones (`TMyEnum = (val1, val2)`)
- Extraer aliases de tipo (`TMyType = Integer`)
- Extraer interfaces (`IMyInterface = interface`)

#### RF-03.3: Extracción de Métodos

- Extraer procedimientos con parámetros
- Extraer funciones con tipo de retorno
- Detectar visibilidad (public, private, protected)
- Extraer número de línea de definición
- Detectar métodos virtuales/override

#### RF-03.4: Extracción de Variables

- Variables globales de unidad
- Constantes con tipo y valor
- Variables de clase/record
- Contar secciones var y bytes estimados

#### RF-03.5: Detección de Llamadas

- Identificar llamadas a funciones/procedimientos
- Detectar llamadas a métodos de otras unidades
- Extraer parámetros de llamadas SQL
- Detectar uso de componentes

#### RF-03.6: Parsing de Formularios (.dfm/.fmx)

- Extraer jerarquía de objetos (name: TClass)
- Extraer propiedades de cada componente
- Detectar DataSource y Dataset references
- Extraer consultas SQL embebidas
- Detectar eventos asignados (OnClick, etc.)

#### RF-03.7: Parsing de Proyectos (.dpr/.dproj)

- Extraer lista de unidades del proyecto
- Detectar configuración de compilación
- Extraer rutas de búsqueda
- Identificar dependencias externas

---

### RF-04: Construcción del Grafo

#### RF-04.1: Generación de Nodos

- Crear nodo por cada archivo de código
- Asignar ID único (hash de ruta)
- Clasificar por tipo (unit, form, component, etc.)
- Calcular tamaño visual (1-12) proporcional
- Almacenar metadata completa en nodo

#### RF-04.2: Generación de Aristas

- Crear arista `uses` por cada dependencia
- Crear arista `pair` entre .pas y .dfm
- Crear arista `calls` por llamadas entre unidades
- Crear arista `inherits` por herencia de clases
- Crear arista `implements` por interfaces

#### RF-04.3: Cálculo de Métricas

- Grado de entrada (in-degree) por nodo
- Grado de salida (out-degree) por nodo
- Desglose por tipo de arista
- Detectar nodos hub (alta conectividad)
- Detectar nodos hoja (sin dependencias salientes)

#### RF-04.4: Detección de Patrones

- Identificar dependencias circulares
- Detectar clusters de módulos relacionados
- Identificar capas arquitectónicas
- Detectar code smells (god classes, etc.)

---

### RF-05: Visualización 3D

#### RF-05.1: Renderizado del Grafo

- Renderizado WebGL via Three.js
- Layout force-directed con física
- Warmup de 150 ticks para estabilización
- Fondo oscuro configurable
- Anti-aliasing habilitado

#### RF-05.2: Representación de Nodos

- Esferas 3D con tamaño variable
- Color por tipo de nodo (configurable)
- Etiqueta de texto flotante
- Highlight al hover
- Glow en nodo seleccionado

#### RF-05.3: Representación de Aristas

- Líneas 3D entre nodos
- Color por tipo de relación
- Grosor configurable (0-3)
- Opacidad configurable (0-1)
- Animación de flujo opcional

#### RF-05.4: Controles de Cámara

- Rotación con click izquierdo + drag
- Pan con click derecho + drag
- Zoom con rueda del mouse
- Zoom con gestos táctiles (pinch)
- Doble click para centrar en nodo
- Botón "Fit" para ver todo el grafo

#### RF-05.5: Optimización de Rendimiento

- LOD automático para >4000 nodos
- Muestreo proporcional por tipo
- Occlusion culling
- Instanced rendering para nodos
- Throttling de actualizaciones

---

### RF-06: Interacción con el Grafo

#### RF-06.1: Selección de Nodos

- Click para seleccionar nodo
- Highlight visual del nodo seleccionado
- Resaltar aristas conectadas
- Deseleccionar con click en vacío
- Selección múltiple con Ctrl+click

#### RF-06.2: Panel de Detalles

- Mostrar al seleccionar nodo
- Información: nombre, ruta, tipo, tamaño
- Lista de dependencias (uses)
- Lista de procedimientos/funciones
- Lista de clases definidas
- Lista de componentes (si es form)
- Métricas de impacto (in/out)
- Botón para abrir archivo en editor externo

#### RF-06.3: Búsqueda de Nodos

- Campo de búsqueda en HUD
- Búsqueda por nombre de archivo
- Búsqueda por nombre de unidad
- Búsqueda por nombre de clase
- Búsqueda por ruta
- Búsqueda con expresiones regulares
- Resultados en lista desplegable
- Click en resultado para navegar

#### RF-06.4: Filtrado de Grafo

- Filtrar por tipo de nodo
- Filtrar por extensión de archivo
- Filtrar por directorio
- Filtrar por rango de tamaño
- Ocultar nodos sin conexiones
- Guardar filtros como presets

#### RF-06.5: Aislamiento de Subgrafo

- Aislar nodo seleccionado + vecinos
- Configurar profundidad (1-4 hops)
- Slider para ajustar profundidad
- Botón para volver al grafo completo
- Animación de transición

---

### RF-07: Análisis de Impacto

#### RF-07.1: Visualización de Impacto

- Mostrar dependencias entrantes (quién me usa)
- Mostrar dependencias salientes (a quién uso)
- Desglose por tipo de relación
- Gráfico de barras en panel lateral
- Ordenar por cantidad de dependencias

#### RF-07.2: Análisis de Propagación

- Simular cambio en un nodo
- Mostrar nodos afectados en cascada
- Calcular profundidad de impacto
- Exportar reporte de impacto

#### RF-07.3: Detección de Riesgos

- Identificar módulos críticos (high coupling)
- Detectar single points of failure
- Alertar sobre dependencias circulares
- Sugerir refactorizaciones

---

### RF-08: Gestión de Versiones

#### RF-08.1: Guardado de Versiones

- Guardar snapshot del análisis
- Asignar nombre/descripción a versión
- Timestamp automático
- Guardar configuración usada

#### RF-08.2: Historial de Versiones

- Listar todas las versiones
- Mostrar fecha y descripción
- Ordenar por fecha
- Eliminar versiones antiguas

#### RF-08.3: Comparación de Versiones

- Seleccionar dos versiones para comparar
- Mostrar nodos agregados/eliminados
- Mostrar aristas nuevas/removidas
- Destacar cambios en métricas
- Vista diff side-by-side

#### RF-08.4: Timeline de Evolución

- Slider temporal entre versiones
- Animación de cambios
- Estadísticas por versión
- Gráfico de evolución de métricas

---

### RF-09: Configuración de Visualización

#### RF-09.1: Temas de Color

- Tema oscuro (default)
- Tema claro
- Tema alto contraste
- Tema daltonismo-friendly
- Temas personalizados

> [!NOTE]
> Este es el theme del style creado con tweekcn para tailwindcss 4
> bunx shadcn@latest add <https://tweakcn.com/r/themes/twitter.json>

#### RF-09.2: Colores por Tipo de Nodo

- Configurar color para cada tipo:
  - unit (azul)
  - form (magenta)
  - component (cian)
  - table (salmón)
  - dataset (amarillo)
  - procedure (verde agua)
  - function (verde)
  - class (violeta)
- Preview en tiempo real
- Reset a valores default

#### RF-09.3: Configuración de Aristas

- Color por tipo de relación
- Grosor global (slider)
- Opacidad global (slider)
- Mostrar/ocultar por tipo
- Animación de flujo on/off

#### RF-09.4: Configuración de Layout

- Seleccionar algoritmo:
  - Force-directed (3D)
  - Hierarchical (árbol)
  - Radial (circular)
  - Grid (cuadrícula)
- Parámetros de física (repulsión, atracción)
- Espaciado entre nodos

#### RF-09.5: Exportar/Importar Configuración

- Exportar tema a archivo JSON
- Importar tema desde archivo
- Compartir temas entre usuarios

---

### RF-10: Exportación de Datos

#### RF-10.1: Exportar Imagen

- Captura PNG del grafo actual
- Resolución configurable (1x, 2x, 4x)
- Fondo transparente opcional
- Incluir leyenda opcional

#### RF-10.2: Exportar SVG

- Vector escalable del grafo
- Preservar colores y etiquetas
- Optimizado para impresión

#### RF-10.3: Exportar Reporte PDF

- Resumen del proyecto
- Métricas principales
- Top 10 módulos críticos
- Gráficos de dependencias
- Lista de warnings/riesgos

#### RF-10.4: Exportar Datos

- JSON del grafo completo
- CSV de nodos con métricas
- CSV de aristas
- Formato DOT (Graphviz)
- Formato GEXF (Gephi)

---

### RF-11: Anotaciones y Documentación

#### RF-11.1: Notas en Nodos

- Agregar nota de texto a cualquier nodo
- Editar/eliminar notas existentes
- Icono indicador de nodo con notas
- Búsqueda en contenido de notas

#### RF-11.2: Etiquetas Personalizadas

- Crear etiquetas/tags
- Asignar etiquetas a nodos
- Filtrar por etiquetas
- Colores personalizados por etiqueta

#### RF-11.3: Estados de Nodo

- Marcar como "Revisar"
- Marcar como "Deprecado"
- Marcar como "Crítico"
- Marcar como "OK"
- Iconos visuales por estado

---

### RF-12: Navegación Avanzada

#### RF-12.1: Minimap

- Vista miniatura del grafo completo
- Indicador de viewport actual
- Click para navegar
- Toggle show/hide

#### RF-12.2: Bookmarks

- Guardar posiciones de cámara
- Nombrar bookmarks
- Lista de acceso rápido
- Atajos de teclado (1-9)

#### RF-12.3: Historial de Navegación

- Back/Forward entre selecciones
- Lista de nodos visitados
- Limpiar historial

#### RF-12.4: Atajos de Teclado

- `Space`: Reset cámara (fit)
- `F`: Centrar en selección
- `Esc`: Deseleccionar
- `Ctrl+F`: Abrir búsqueda
- `Ctrl+G`: Toggle grid
- `1-4`: Cambiar layout
- `+/-`: Zoom in/out
- `←→↑↓`: Mover cámara

---

### RF-13: Conexión a Base de Datos (Opcional)

#### RF-13.1: Configuración de Conexión

- Tipo: MySQL, MariaDB, PostgreSQL, SQLite
- Host y puerto
- Usuario y contraseña
- Nombre de base de datos
- Probar conexión

#### RF-13.2: Análisis de Esquema

- Listar tablas
- Extraer columnas y tipos
- Detectar foreign keys
- Detectar índices

#### RF-13.3: Grafo de BD

- Nodos para tablas
- Aristas para foreign keys
- Integrar con grafo de código
- Detectar queries en código que usan tablas

---

### RF-14: Multi-lenguaje (Futuro)

#### RF-14.1: Parsers Adicionales

- C# (.cs)
- Java (.java)
- Python (.py)
- TypeScript (.ts/.tsx)
- Go (.go)

#### RF-14.2: Detección Automática

- Detectar lenguaje por extensión
- Detectar lenguaje por contenido
- Proyectos multi-lenguaje

---

## REQUERIMIENTOS NO FUNCIONALES

### RNF-01: Rendimiento

- Tiempo de escaneo: <30s para 5000 archivos
- Tiempo de parsing: <60s para 5000 archivos
- FPS del grafo: >30fps con 3000 nodos
- Memoria máxima: <2GB RAM
- Startup time: <3 segundos

### RNF-02: Usabilidad

- Interfaz intuitiva sin necesidad de manual
- Tooltips en todos los controles
- Feedback visual en operaciones largas
- Mensajes de error claros y accionables
- Undo/Redo para acciones destructivas

### RNF-03: Accesibilidad

- Soporte para lectores de pantalla
- Navegación completa por teclado
- Contraste mínimo WCAG AA
- Tamaños de fuente configurables
- Modo alto contraste

### RNF-04: Seguridad

- Sin conexión a internet requerida
- Datos almacenados localmente
- Sin telemetría ni tracking
- Archivos de proyecto encriptables (opcional)

### RNF-05: Mantenibilidad

- Código modular y tipado
- Tests unitarios >80% cobertura
- Documentación inline (JSDoc/rustdoc)
- CI/CD automatizado
- Semantic versioning

### RNF-06: Internacionalización

- Soporte para español (default)
- Soporte para inglés
- Arquitectura preparada para más idiomas
- Fechas y números localizados

---

## ARQUITECTURA

```
devwaresoft-v2/
├── src-tauri/                    # Backend Rust
│   ├── src/
│   │   ├── main.rs              # Entry point
│   │   ├── lib.rs               # Library exports
│   │   ├── commands/            # Tauri commands (IPC)
│   │   │   ├── mod.rs
│   │   │   ├── project.rs       # Gestión de proyectos
│   │   │   ├── scan.rs          # Escaneo de archivos
│   │   │   ├── parse.rs         # Parsing de código
│   │   │   ├── graph.rs         # Construcción de grafo
│   │   │   └── config.rs        # Configuración
│   │   ├── parsers/             # Parsers por lenguaje
│   │   │   ├── mod.rs
│   │   │   ├── delphi/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── pas.rs       # Parser .pas
│   │   │   │   ├── dfm.rs       # Parser .dfm
│   │   │   │   └── dpr.rs       # Parser .dpr
│   │   │   └── traits.rs        # Parser trait
│   │   ├── graph/               # Motor de grafo
│   │   │   ├── mod.rs
│   │   │   ├── node.rs
│   │   │   ├── edge.rs
│   │   │   ├── builder.rs
│   │   │   └── metrics.rs
│   │   ├── storage/             # Persistencia
│   │   │   ├── mod.rs
│   │   │   ├── project.rs
│   │   │   ├── json.rs
│   │   │   └── sqlite.rs
│   │   └── utils/
│   │       ├── mod.rs
│   │       ├── hash.rs
│   │       └── zip.rs
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/                          # Frontend React
│   ├── main.tsx                 # Entry point
│   ├── App.tsx                  # Root component
│   ├── components/
│   │   ├── ui/                  # shadcn/ui components
│   │   │   ├── button.tsx
│   │   │   ├── input.tsx
│   │   │   ├── dialog.tsx
│   │   │   ├── slider.tsx
│   │   │   ├── tabs.tsx
│   │   │   └── ...
│   │   ├── layout/
│   │   │   ├── Header.tsx
│   │   │   ├── Sidebar.tsx
│   │   │   └── StatusBar.tsx
│   │   ├── graph/
│   │   │   ├── Graph3D.tsx      # Three.js canvas
│   │   │   ├── Node.tsx         # Componente nodo
│   │   │   ├── Edge.tsx         # Componente arista
│   │   │   ├── Controls.tsx     # Controles de cámara
│   │   │   └── Minimap.tsx
│   │   ├── panels/
│   │   │   ├── NodeDetails.tsx
│   │   │   ├── SearchPanel.tsx
│   │   │   ├── FilterPanel.tsx
│   │   │   └── SettingsPanel.tsx
│   │   └── dialogs/
│   │       ├── NewProject.tsx
│   │       ├── ScanProgress.tsx
│   │       └── ExportDialog.tsx
│   ├── hooks/
│   │   ├── useGraph.ts
│   │   ├── useCamera.ts
│   │   ├── useSelection.ts
│   │   ├── useSearch.ts
│   │   └── useTauri.ts
│   ├── store/
│   │   ├── index.ts
│   │   ├── graphStore.ts
│   │   ├── uiStore.ts
│   │   ├── projectStore.ts
│   │   └── settingsStore.ts
│   ├── lib/
│   │   ├── tauri.ts             # Wrapper comandos Tauri
│   │   ├── three/
│   │   │   ├── materials.ts
│   │   │   ├── geometries.ts
│   │   │   └── effects.ts
│   │   └── utils.ts
│   ├── types/
│   │   ├── graph.ts
│   │   ├── project.ts
│   │   └── settings.ts
│   ├── themes/
│   │   ├── dark.ts
│   │   ├── light.ts
│   │   └── index.ts
│   └── i18n/
│       ├── es.json
│       └── en.json
│
├── public/
│   └── icons/
├── package.json
├── tailwind.config.ts
├── tsconfig.json
├── vite.config.ts
└── README.md
```

---

## DISEÑO DE UI

### Layout Principal

```
┌─────────────────────────────────────────────────────────────────┐
│  [Logo] DevpWareSoft    [Search...]    [⚙️] [🌙] [?]           │
├────────────┬────────────────────────────────────────────────────┤
│            │                                                    │
│  Projects  │                                                    │
│  ────────  │                                                    │
│  > Proj 1  │                                                    │
│    Proj 2  │              GRAFO 3D                              │
│            │                                                    │
│  Filters   │                                                    │
│  ────────  │                                                    │
│  ☑ Units   │                                                    │
│  ☑ Forms   │                                                    │
│  ☐ Tables  │                                                    │
│            │                                                    │
│  Layout    │                                     ┌─────────┐    │
│  ────────  │                                     │ Minimap │    │
│  ◉ Force   │                                     └─────────┘    │
│  ○ Tree    │                                                    │
│  ○ Radial  │                                                    │
│            │                                                    │
├────────────┴────────────────────────────────────────────────────┤
│  Nodes: 1,234  │  Edges: 5,678  │  Selected: FormMain.pas      │
└─────────────────────────────────────────────────────────────────┘
```

### Panel de Nodo Seleccionado

```
┌──────────────────────────────┐
│ FormMain.pas            [×]  │
├──────────────────────────────┤
│ Type: unit                   │
│ Path: src/UI/FormMain.pas    │
│ Size: 24.5 KB                │
├──────────────────────────────┤
│ Dependencies (15)        [▼] │
│ ├─ Windows                   │
│ ├─ SysUtils                  │
│ ├─ Classes                   │
│ └─ ...more                   │
├──────────────────────────────┤
│ Methods (8)              [▼] │
│ ├─ procedure FormCreate      │
│ ├─ procedure ButtonClick     │
│ └─ ...more                   │
├──────────────────────────────┤
│ Impact                       │
│ In: ████████░░ 42            │
│ Out: ███░░░░░░ 15            │
├──────────────────────────────┤
│ [Open in Editor] [Isolate]   │
└──────────────────────────────┘
```

---

## ANIMACIONES (Framer Motion)

### Transiciones de Página

```typescript
const pageVariants = {
  initial: { opacity: 0, x: -20 },
  animate: { opacity: 1, x: 0 },
  exit: { opacity: 0, x: 20 },
};
```

### Panel Slide

```typescript
const panelVariants = {
  closed: { width: 0, opacity: 0 },
  open: { width: 320, opacity: 1 },
};
```

### Nodo Hover

```typescript
const nodeHover = {
  scale: 1.2,
  filter: "brightness(1.3)",
  transition: { duration: 0.2 },
};
```

### Loading States

```typescript
const pulseAnimation = {
  opacity: [0.5, 1, 0.5],
  transition: { repeat: Infinity, duration: 1.5 },
};
```

---

## COMANDOS TAURI (IPC)

```typescript
// Definición de comandos disponibles
interface TauriCommands {
  // Proyectos
  create_project: (name: string, path: string) => Project;
  open_project: (id: string) => Project;
  list_projects: () => Project[];
  delete_project: (id: string) => void;

  // Escaneo
  scan_directory: (path: string, config: ScanConfig) => ScanResult;
  scan_zip: (path: string) => ScanResult;
  cancel_scan: () => void;

  // Parsing
  parse_project: (scanResult: ScanResult) => ParseResult;

  // Grafo
  build_graph: (parseResult: ParseResult) => Graph;
  get_graph: (projectId: string, version?: string) => Graph;
  get_impact: (nodeId: string) => ImpactData;

  // Versiones
  save_version: (projectId: string, name: string) => Version;
  list_versions: (projectId: string) => Version[];
  load_version: (projectId: string, versionId: string) => Graph;

  // Configuración
  get_settings: () => Settings;
  save_settings: (settings: Settings) => void;

  // Exportación
  export_png: (options: ExportOptions) => string; // path
  export_pdf: (options: ExportOptions) => string;
  export_json: (projectId: string) => string;

  // Utilidades
  open_file_dialog: (filters: FileFilter[]) => string | null;
  open_folder_dialog: () => string | null;
  open_in_editor: (path: string) => void;
}
```

---

## MODELO DE DATOS

### TypeScript Types

```typescript
// Nodo del grafo
interface GraphNode {
  id: string;
  type: NodeType;
  name: string;
  label: string;
  size: number; // 1-12
  position?: { x: number; y: number; z: number };
  meta: NodeMeta;
}

type NodeType =
  | "unit"
  | "form"
  | "component"
  | "datamodule"
  | "table"
  | "dataset"
  | "query"
  | "class"
  | "interface"
  | "record"
  | "procedure"
  | "function";

interface NodeMeta {
  path: string;
  absolutePath: string;
  extension: string;
  sizeBytes: number;
  layer?: string;
  group?: string;
  unit?: string;
  uses?: string[];
  methods?: Method[];
  classes?: ClassDef[];
  variables?: VariableInfo;
  components?: Component[];
  properties?: Property[];
  notes?: string;
  tags?: string[];
  status?: NodeStatus;
}

// Arista del grafo
interface GraphEdge {
  id: string;
  source: string;
  target: string;
  type: EdgeType;
  detail?: string;
}

type EdgeType =
  | "uses"
  | "pair"
  | "calls"
  | "inherits"
  | "implements"
  | "contains";

// Grafo completo
interface Graph {
  nodes: GraphNode[];
  edges: GraphEdge[];
  metadata: GraphMetadata;
}

// Proyecto
interface Project {
  id: string;
  name: string;
  description?: string;
  rootPath: string;
  language: "delphi" | "csharp" | "java" | "python";
  createdAt: string;
  updatedAt: string;
  currentVersion?: string;
  config: ProjectConfig;
}

// Configuración de tema
interface ThemeConfig {
  id: string;
  name: string;
  isDark: boolean;
  colors: {
    background: string;
    surface: string;
    text: string;
    textMuted: string;
    border: string;
    accent: string;
    nodes: Record<NodeType, string>;
    edges: Record<EdgeType, string>;
  };
  graph: {
    nodeOpacity: number;
    edgeOpacity: number;
    edgeWidth: number;
    labelSize: number;
    glowIntensity: number;
  };
}
```

---

## ROADMAP DE IMPLEMENTACIÓN

### Fase 1: Fundación (2-3 semanas)

- [ ] Setup proyecto Tauri + React + TypeScript
- [ ] Configurar Tailwind + shadcn/ui
- [ ] Implementar layout principal
- [ ] Setup Zustand stores básicos
- [ ] Crear comandos Tauri básicos (file dialogs)

### Fase 2: Core Backend (2-3 semanas)

- [ ] Implementar scanner de directorios en Rust
- [ ] Implementar parser Delphi .pas
- [ ] Implementar parser Delphi .dfm
- [ ] Implementar builder de grafo
- [ ] Implementar persistencia JSON

### Fase 3: Visualización (2-3 semanas)

- [ ] Integrar Three.js / React Three Fiber
- [ ] Implementar renderizado de nodos
- [ ] Implementar renderizado de aristas
- [ ] Implementar controles de cámara
- [ ] Implementar layout force-directed

### Fase 4: Interacción (1-2 semanas)

- [ ] Implementar selección de nodos
- [ ] Implementar panel de detalles
- [ ] Implementar búsqueda
- [ ] Implementar filtros básicos

### Fase 5: Features Avanzadas (2-3 semanas)

- [ ] Sistema de temas y colores
- [ ] Gestión de versiones
- [ ] Exportación (PNG, PDF, JSON)
- [ ] Análisis de impacto avanzado

### Fase 6: Pulido (1-2 semanas)

- [ ] Animaciones Framer Motion
- [ ] Minimap
- [ ] Atajos de teclado
- [ ] Optimización de rendimiento
- [ ] Testing y bugs

---

## COMANDOS DE DESARROLLO

```bash
# Instalar dependencias
bun install

# Desarrollo
bun tauri dev

# Build para producción
bun tauri build

# Lint
bun lint

# Tests
bun test

# Generar componentes shadcn
bun dlx shadcn@latest add button input dialog
```

---

## VERIFICACIÓN

Para validar que la migración está completa:

1. **Funcionalidad Core**
   - [ ] Escanear proyecto Delphi existente
   - [ ] Verificar que se detectan todos los archivos
   - [ ] Verificar parsing correcto de .pas y .dfm
   - [ ] Comparar grafo generado con versión anterior

2. **Visualización**
   - [ ] Grafo se renderiza correctamente
   - [ ] Colores por tipo funcionan
   - [ ] Zoom/pan/rotación funcionan
   - [ ] Selección de nodos funciona

3. **Rendimiento**
   - [ ] Cargar grafo de 3000+ nodos sin lag
   - [ ] FPS estable >30
   - [ ] Memoria <2GB

4. **Multiplataforma**
   - [ ] Build en Windows
   - [ ] Build en macOS
   - [ ] Build en Linux
   - [ ] Instaladores funcionan
