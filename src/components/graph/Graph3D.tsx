import { Canvas, useFrame, ThreeEvent } from "@react-three/fiber";
import { OrbitControls, PerspectiveCamera, Html } from "@react-three/drei";
import { cn } from "@/lib/utils";
import { useGraphStore } from "@/store/graphStore";
import { useSettingsStore } from "@/store/settingsStore";
import { Suspense, useRef, useMemo, useCallback } from "react";
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
  isConnected: boolean;
  onHover: (node: UnifiedNode | null) => void;
  color: string;
  sizeMultiplier: number;
}

function NodeMesh({ node, onSelect, isSelected, isHovered, isConnected, onHover, color, sizeMultiplier }: NodeMeshProps) {
  const position = node.position ?? { x: 0, y: 0, z: 0 };

  const scale = isSelected ? 1.8 : isHovered ? 1.4 : isConnected ? 1.2 : 1;
  const emissiveIntensity = isSelected ? 1 : isHovered ? 0.6 : isConnected ? 0.4 : 0.15;

  const handleClick = useCallback((e: ThreeEvent<MouseEvent>) => {
    e.stopPropagation();
    onSelect(node);
  }, [node, onSelect]);

  const handlePointerOver = useCallback((e: ThreeEvent<PointerEvent>) => {
    e.stopPropagation();
    onHover(node);
    document.body.style.cursor = "pointer";
  }, [node, onHover]);

  const handlePointerOut = useCallback(() => {
    onHover(null);
    document.body.style.cursor = "default";
  }, [onHover]);

  const nodeSize = node.size * 0.08 * sizeMultiplier;

  return (
    <group position={[position.x, position.y, position.z]}>
      <mesh
        scale={scale}
        onClick={handleClick}
        onPointerOver={handlePointerOver}
        onPointerOut={handlePointerOut}
      >
        <sphereGeometry args={[nodeSize, 12, 12]} />
        <meshStandardMaterial
          color={color}
          emissive={color}
          emissiveIntensity={emissiveIntensity}
          roughness={0.3}
          metalness={0.5}
        />
      </mesh>

      {/* Glow ring for selected */}
      {isSelected && (
        <mesh rotation={[Math.PI / 2, 0, 0]}>
          <ringGeometry args={[nodeSize * 1.8, nodeSize * 2.2, 24]} />
          <meshBasicMaterial
            color={color}
            transparent
            opacity={0.5}
            side={THREE.DoubleSide}
          />
        </mesh>
      )}

      {/* Label */}
      {(isSelected || isHovered) && (
        <Html
          position={[0, nodeSize * 2, 0]}
          center
          style={{ pointerEvents: "none", whiteSpace: "nowrap" }}
        >
          <div className="bg-black/90 text-white px-2 py-1 rounded text-xs border border-white/30 shadow-xl">
            {node.name}
          </div>
        </Html>
      )}
    </group>
  );
}

function EdgeLines({ selectedNodeId }: { selectedNodeId: string | null }) {
  const { graph, filteredNodes } = useGraphStore();

  const lineData = useMemo(() => {
    if (!selectedNodeId || !graph) return null;

    const nodes = filteredNodes();
    const nodePositions = new Map<string, THREE.Vector3>();
    nodes.forEach((node) => {
      const pos = node.position ?? { x: 0, y: 0, z: 0 };
      nodePositions.set(node.id, new THREE.Vector3(pos.x, pos.y, pos.z));
    });

    const connectedEdges = graph.edges.filter(
      (e) => e.source === selectedNodeId || e.target === selectedNodeId
    );

    const points: THREE.Vector3[] = [];
    connectedEdges.forEach((edge) => {
      const startPos = nodePositions.get(edge.source);
      const endPos = nodePositions.get(edge.target);
      if (startPos && endPos) {
        points.push(startPos.clone(), endPos.clone());
      }
    });

    if (points.length === 0) return null;

    const geometry = new THREE.BufferGeometry().setFromPoints(points);
    return geometry;
  }, [selectedNodeId, graph, filteredNodes]);

  if (!lineData) return null;

  return (
    <lineSegments geometry={lineData}>
      <lineBasicMaterial color="#00ff88" transparent opacity={0.6} />
    </lineSegments>
  );
}

function RotatingGroup({ children, speed }: { children: React.ReactNode; speed: number }) {
  const groupRef = useRef<THREE.Group>(null);

  useFrame((_, delta) => {
    if (groupRef.current && speed > 0) {
      groupRef.current.rotation.y += delta * speed;
    }
  });

  return <group ref={groupRef}>{children}</group>;
}

function GraphScene() {
  const { filteredNodes, selectNode, selectedNode, hoveredNode, hoverNode, graph } =
    useGraphStore();
  const { nodeColors, rotationSpeed, nodeSize } = useSettingsStore();
  const nodes = filteredNodes();

  const connectedNodeIds = useMemo(() => {
    if (!selectedNode || !graph) return new Set<string>();
    const ids = new Set<string>();
    graph.edges.forEach((edge) => {
      if (edge.source === selectedNode.id) ids.add(edge.target);
      if (edge.target === selectedNode.id) ids.add(edge.source);
    });
    return ids;
  }, [selectedNode, graph]);

  const getNodeColor = useCallback(
    (nodeType: string | { custom: string }): string => {
      const type = typeof nodeType === "string" ? nodeType : "custom";
      return nodeColors[type] || nodeColors.custom || "#6b7280";
    },
    [nodeColors]
  );

  return (
    <>
      <ambientLight intensity={0.4} />
      <pointLight position={[100, 100, 100]} intensity={0.6} />
      <pointLight position={[-100, -100, -100]} intensity={0.3} color="#4488ff" />

      <RotatingGroup speed={rotationSpeed}>
        {/* Only show edges for selected node */}
        <EdgeLines selectedNodeId={selectedNode?.id ?? null} />

        {/* Nodes */}
        {nodes.map((node) => (
          <NodeMesh
            key={node.id}
            node={node}
            onSelect={selectNode}
            isSelected={selectedNode?.id === node.id}
            isHovered={hoveredNode?.id === node.id}
            isConnected={connectedNodeIds.has(node.id)}
            onHover={hoverNode}
            color={getNodeColor(node.node_type)}
            sizeMultiplier={nodeSize}
          />
        ))}
      </RotatingGroup>
    </>
  );
}

function LoadingFallback() {
  const meshRef = useRef<THREE.Mesh>(null);

  useFrame((_, delta) => {
    if (meshRef.current) {
      meshRef.current.rotation.x += delta * 0.5;
      meshRef.current.rotation.y += delta * 0.3;
    }
  });

  return (
    <mesh ref={meshRef}>
      <torusKnotGeometry args={[1, 0.3, 64, 12]} />
      <meshStandardMaterial color="#1e9df1" wireframe />
    </mesh>
  );
}

function EmptyState() {
  const meshRef = useRef<THREE.Mesh>(null);

  useFrame((_, delta) => {
    if (meshRef.current) {
      meshRef.current.rotation.y += delta * 0.1;
    }
  });

  return (
    <mesh ref={meshRef}>
      <torusKnotGeometry args={[3, 0.8, 128, 24]} />
      <meshStandardMaterial
        color="#1e9df1"
        wireframe
        transparent
        opacity={0.4}
      />
    </mesh>
  );
}

export function Graph3D({ className }: Graph3DProps) {
  const { graph } = useGraphStore();

  return (
    <div className={cn("w-full h-full", className)}>
      <Canvas gl={{ antialias: true, alpha: true, powerPreference: "high-performance" }}>
        <color attach="background" args={["#0a0a0f"]} />
        <fog attach="fog" args={["#0a0a0f", 80, 250]} />

        <PerspectiveCamera makeDefault position={[0, 0, 80]} fov={60} />
        <OrbitControls
          enableDamping
          dampingFactor={0.05}
          rotateSpeed={0.5}
          zoomSpeed={0.8}
          minDistance={20}
          maxDistance={200}
        />

        <Suspense fallback={<LoadingFallback />}>
          {graph && graph.nodes.length > 0 ? <GraphScene /> : <EmptyState />}
        </Suspense>
      </Canvas>
    </div>
  );
}
