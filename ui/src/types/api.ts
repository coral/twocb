// Mirrors ts-rs generated types from files/types/

export type ParamKind = "Slider" | "HsvPicker" | "RgbPicker" | "Toggle" | "Number";

export type ParamValue =
  | { Float: number }
  | { Color3: [number, number, number] }
  | { Bool: boolean };

export interface PatternParam {
  name: string;
  kind: ParamKind;
  value: ParamValue;
}

export interface Step {
  pattern: string;
  engine_type: string;
  blendmode: string;
}

export interface Link {
  name: string;
  opacity: number;
  steps: Step[];
}

export interface Order {
  reference: number;
  name: string;
}

export interface Endpoint {
  start: number;
  end: number;
  protocol: "opc" | "ddp";
  host: string;
  port: number;
}

export interface Config {
  fps: number;
  endpoints: Endpoint[];
  audio: {
    sampleRate: number;
    bufferSize: number;
    channels: number;
    tempo: { confidenceLimit: number };
  };
  database: string;
  api: { host: string; port: number };
  mapping: string;
}

export interface Pixel {
  I: number;
  A: boolean;
  O: [number, number, number];
  N: [number, number, number];
}
