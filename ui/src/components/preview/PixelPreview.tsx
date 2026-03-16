import { Canvas } from "@react-three/fiber";
import { OrbitControls } from "@react-three/drei";
import PixelCloud from "./PixelCloud";
import { usePixels } from "../../stores/pixels";

export default function PixelPreview() {
  const pixelCount = usePixels((s) => s.pixelCount);

  return (
    <div className="w-full h-full bg-black">
      <Canvas
        camera={{ position: [1.5, 1.5, 1.5], fov: 50, near: 0.01, far: 100 }}
        gl={{ antialias: true }}
      >
        <color attach="background" args={["#050505"]} />
        <OrbitControls makeDefault enableDamping dampingFactor={0.1} />
        {pixelCount > 0 && <PixelCloud />}
      </Canvas>
    </div>
  );
}
