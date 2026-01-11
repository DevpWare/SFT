import { Canvas } from "@react-three/fiber";
import { OrbitControls, PerspectiveCamera, Html } from "@react-three/drei";
import { cn } from "@/lib/utils";
import { useGraphStore } from "@/store/graphStore";
import { Suspense, useRef } from "react";
import type { UnifiedNode } from "@/types/unified";
import * as THREE from "three";

interface Graph3DProps {
  className?: string;
}

interface NodeMeshProps {
  node: UnifiedNode;
  onSelect: (node: UnifiedNode) => void;
  isSelected: boolean;
  isHovered: boolean;
  onHover: (node: UnifiedNode | null) => void;
}

function NodeMesh({ node, onSelect, isSelected, isHovered, onHover }: NodeMeshProps) {
  const meshRef = useRef<THREE.Mesh>(null);
  const color = getNodeColor(node.node_type);
  const position = node.position ?? { x: 0, y: 0, z: 0 };

  return (
    <group position={[position.x, position.y, position.z]}>
      <mesh
        ref={meshRef}
        onClick={(e) => {
          e.stopPropagation();
          onSelect(node);
        }}
        onPointerOver={(e) => {
          e.stopPropagation();
          onHover(node);
          document.body.style.cursor = "pointer";
        }}
        onPointerOut={() => {
          onHover(null);
          document.body.style.cursor = "default";
        }}
      >
        <sphereGeometry args={[node.size * 0.08, 24, 24]} />
        <meshStandardMaterial
          color={color}
          emissive={color}
          emissiveIntensity={isSelected ? 0.5 : isHovered ? 0.3 : 0.1}
          roughness={0.4}
          metalness={0.3}
        />
      </mesh>

      {/* Glow effect for selected/hovered */}
      {(isSelected || isHovered) && (
        <mesh>
          <sphereGeometry args={[node.size * 0.12, 16, 16]} />
          <meshBasicMaterial
            color={color}
            transparent
            opacity={isSelected ? 0.3 : 0.15}
          />
        </mesh>
      )}

      {/* Label */}
      {(isSelected || isHovered) && (
        <Html
          position={[0, node.size * 0.15, 0]}
          center
          style={{
            pointerEvents: "none",
            whiteSpace: "nowrap",
          }}
        >
          <div className="bg-popover/90 backdrop-blur-sm text-popover-foreground px-2 py-1 rounded text-xs border border-border shadow-lg">
            {node.name}
          </div>
        </Html>
      )}
    </group>
  );
}

function GraphScene() {
  const { filteredNodes, selectNode, selectedNode, hoveredNode, hoverNode } = useGraphStore();
  const nodes = filteredNodes();

  return (
    <>
      <ambientLight intensity={0.4} />
      <pointLight position={[10, 10, 10]} intensity={0.8} />
      <pointLight position={[-10, -10, -10]} intensity={0.4} />

      {nodes.map((node) => (
        <NodeMesh
          key={node.id}
          node={node}
          onSelect={selectNode}
          isSelected={selectedNode?.id === node.id}
          isHovered={hoveredNode?.id === node.id}
          onHover={hoverNode}
        />
      ))}
    </>
  );
}

function getNodeColor(nodeType: string | { custom: string }): string {
  const type = typeof nodeType === "string" ? nodeType : "custom";

  const colors: Record<string, string> = {
    module: "#1e9df1",
    form: "#e91e63",
    class: "#9c27b0",
    function: "#17bf63",
    method: "#00b87a",
    component: "#00bcd4",
    source_file: "#72767a",
    controller: "#ff2d20",
    model: "#f7b928",
    view: "#9b59b6",
    route: "#f39c12",
    interface: "#68217a",
    custom: "#6b7280",
  };

  return colors[type] ?? colors.custom;
}

function LoadingFallback() {
  return (
    <mesh>
      <torusKnotGeometry args={[1, 0.3, 128, 16]} />
      <meshStandardMaterial color="#1e9df1" wireframe />
    </mesh>
  );
}

function EmptyState() {
  return (
    <group>
      <mesh rotation={[0, 0, 0]}>
        <torusKnotGeometry args={[2, 0.5, 128, 16]} />
        <meshStandardMaterial
          color="#1e9df1"
          wireframe
          transparent
          opacity={0.5}
        />
      </mesh>
    </group>
  );
}

export function Graph3D({ className }: Graph3DProps) {
  const { graph, selectNode } = useGraphStore();

  return (
    <div className={cn("flex-1 bg-background relative", className)}>
      <Canvas
        onClick={() => selectNode(null)}
        gl={{ antialias: true }}
      >
        <color attach="background" args={["#000000"]} />
        <fog attach="fog" args={["#000000", 30, 100]} />

        <PerspectiveCamera makeDefault position={[0, 0, 40]} fov={60} />
        <OrbitControls
          enableDamping
          dampingFactor={0.05}
          rotateSpeed={0.5}
          zoomSpeed={0.8}
          minDistance={5}
          maxDistance={100}
        />

        <Suspense fallback={<LoadingFallback />}>
          {graph && graph.nodes.length > 0 ? <GraphScene /> : <EmptyState />}
        </Suspense>
      </Canvas>

      {/* Overlay for empty state */}
      {(!graph || graph.nodes.length === 0) && (
        <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
          <div className="text-center space-y-2">
            <p className="text-xl text-muted-foreground">
              Open a project to visualize dependencies
            </p>
            <p className="text-sm text-muted-foreground/60">
              Click "Open Project" in the sidebar
            </p>
          </div>
        </div>
      )}

      {/* Legend */}
      {graph && graph.nodes.length > 0 && (
        <div className="absolute bottom-4 right-4 bg-card/80 backdrop-blur-sm border rounded-lg p-3 text-xs">
          <div className="font-medium mb-2 text-card-foreground">Node Types</div>
          <div className="space-y-1">
            {[
              { type: "module", color: "#1e9df1" },
              { type: "form", color: "#e91e63" },
              { type: "component", color: "#00bcd4" },
              { type: "source_file", color: "#72767a" },
            ].map(({ type, color }) => (
              <div key={type} className="flex items-center gap-2">
                <div
                  className="w-3 h-3 rounded-full"
                  style={{ backgroundColor: color }}
                />
                <span className="capitalize text-muted-foreground">{type.replace(/_/g, " ")}</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}
