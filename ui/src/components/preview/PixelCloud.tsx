import { useRef, useMemo } from "react";
import { useFrame } from "@react-three/fiber";
import * as THREE from "three";
import { usePixels } from "../../stores/pixels";

export default function PixelCloud() {
  const mapping = usePixels((s) => s.mapping);
  const colorsRef = useRef(usePixels.getState().colors);
  const geomRef = useRef<THREE.BufferGeometry>(null);

  // Build static position buffer from mapping
  const positions = useMemo(() => {
    const arr = new Float32Array(mapping.length * 3);
    for (let i = 0; i < mapping.length; i++) {
      arr[i * 3] = mapping[i].O[0];
      arr[i * 3 + 1] = mapping[i].O[1];
      arr[i * 3 + 2] = mapping[i].O[2];
    }
    return arr;
  }, [mapping]);

  // Initial color buffer (black)
  const initialColors = useMemo(() => new Float32Array(mapping.length * 3), [mapping]);

  // Update color buffer every frame from the store (no React re-render)
  useFrame(() => {
    const geom = geomRef.current;
    if (!geom) return;
    const colorAttr = geom.getAttribute("color") as THREE.BufferAttribute;
    const storeColors = usePixels.getState().colors;
    if (storeColors.length === colorAttr.array.length) {
      (colorAttr.array as Float32Array).set(storeColors);
      colorAttr.needsUpdate = true;
    }
  });

  return (
    <points>
      <bufferGeometry ref={geomRef}>
        <bufferAttribute
          attach="attributes-position"
          args={[positions, 3]}
        />
        <bufferAttribute
          attach="attributes-color"
          args={[initialColors, 3]}
        />
      </bufferGeometry>
      <pointsMaterial
        size={0.015}
        vertexColors
        sizeAttenuation
        transparent
        opacity={0.9}
        blending={THREE.AdditiveBlending}
        depthWrite={false}
      />
    </points>
  );
}
