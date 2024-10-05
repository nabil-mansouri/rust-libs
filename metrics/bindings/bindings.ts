// Auto-generated with deno_bindgen
function encode(v: string | Uint8Array): Uint8Array {
  if (typeof v !== "string") return v;
  return new TextEncoder().encode(v);
}

function decode(v: Uint8Array): string {
  return new TextDecoder().decode(v);
}

// deno-lint-ignore no-explicit-any
function readPointer(v: any): Uint8Array {
  const ptr = new Deno.UnsafePointerView(v);
  const lengthBe = new Uint8Array(4);
  const view = new DataView(lengthBe.buffer);
  ptr.copyInto(lengthBe, 0);
  const buf = new Uint8Array(view.getUint32(0));
  ptr.copyInto(buf, 4);
  return buf;
}

const url = new URL("../target/release", import.meta.url);

let uri = url.pathname;
if (!uri.endsWith("/")) uri += "/";

// https://docs.microsoft.com/en-us/windows/win32/api/libloaderapi/nf-libloaderapi-loadlibrarya#parameters
if (Deno.build.os === "windows") {
  uri = uri.replace(/\//g, "\\");
  // Remove leading slash
  if (uri.startsWith("\\")) {
    uri = uri.slice(1);
  }
}

const { symbols } = Deno.dlopen(
  {
    darwin: uri + "libmetrics.dylib",
    windows: uri + "metrics.dll",
    linux: uri + "libmetrics.so",
    freebsd: uri + "libmetrics.so",
    netbsd: uri + "libmetrics.so",
    aix: uri + "libmetrics.so",
    solaris: uri + "libmetrics.so",
    illumos: uri + "libmetrics.so",
  }[Deno.build.os],
  {
    get_cert: {
      parameters: ["buffer", "usize", "u32", "u8"],
      result: "buffer",
      nonblocking: true,
    },
    get_cpu_stime: { parameters: ["u32"], result: "u64", nonblocking: false },
    get_cpu_usage: { parameters: ["u32"], result: "f32", nonblocking: false },
    get_cpu_utime: { parameters: ["u32"], result: "u64", nonblocking: false },
    get_max_children: { parameters: [], result: "i64", nonblocking: false },
    get_memory: { parameters: ["u32"], result: "u64", nonblocking: false },
    get_process_info: {
      parameters: ["u32"],
      result: "buffer",
      nonblocking: false,
    },
    get_process_time: {
      parameters: ["u32"],
      result: "buffer",
      nonblocking: false,
    },
    get_run_time: { parameters: ["u32"], result: "u64", nonblocking: false },
    get_start_time: { parameters: ["u32"], result: "u64", nonblocking: false },
    get_virtual_memory: {
      parameters: ["u32"],
      result: "u64",
      nonblocking: false,
    },
    verify_signature: {
      parameters: ["buffer", "usize", "buffer", "usize", "buffer", "usize"],
      result: "buffer",
      nonblocking: true,
    },
  },
);
export type CertDetails = {
  certificat: string;
  public_key: string;
  error: string;
};
export type ProcessInfo = {
  cpu_usage: number;
  start_time: number;
  run_time: number;
  virtual_memory: number;
  memory: number;
};
export type ProcessTime = {
  user_time: number;
  system_time: number;
};
export function get_cert(a0: string, a1: number, a2: number) {
  const a0_buf = encode(a0);

  const rawResult = symbols.get_cert(a0_buf, a0_buf.byteLength, a1, a2);
  const result = rawResult.then(readPointer);
  return result.then((r) => JSON.parse(decode(r))) as Promise<CertDetails>;
}
export function get_cpu_stime(a0: number) {
  const rawResult = symbols.get_cpu_stime(a0);
  const result = rawResult;
  return result;
}
export function get_cpu_usage(a0: number) {
  const rawResult = symbols.get_cpu_usage(a0);
  const result = rawResult;
  return result;
}
export function get_cpu_utime(a0: number) {
  const rawResult = symbols.get_cpu_utime(a0);
  const result = rawResult;
  return result;
}
export function get_max_children() {
  const rawResult = symbols.get_max_children();
  const result = rawResult;
  return result;
}
export function get_memory(a0: number) {
  const rawResult = symbols.get_memory(a0);
  const result = rawResult;
  return result;
}
export function get_process_info(a0: number) {
  const rawResult = symbols.get_process_info(a0);
  const result = readPointer(rawResult);
  return JSON.parse(decode(result)) as ProcessInfo;
}
export function get_process_time(a0: number) {
  const rawResult = symbols.get_process_time(a0);
  const result = readPointer(rawResult);
  return JSON.parse(decode(result)) as ProcessTime;
}
export function get_run_time(a0: number) {
  const rawResult = symbols.get_run_time(a0);
  const result = rawResult;
  return result;
}
export function get_start_time(a0: number) {
  const rawResult = symbols.get_start_time(a0);
  const result = rawResult;
  return result;
}
export function get_virtual_memory(a0: number) {
  const rawResult = symbols.get_virtual_memory(a0);
  const result = rawResult;
  return result;
}
export function verify_signature(a0: string, a1: Uint8Array, a2: Uint8Array) {
  const a0_buf = encode(a0);
  const a1_buf = encode(a1);
  const a2_buf = encode(a2);

  const rawResult = symbols.verify_signature(
    a0_buf,
    a0_buf.byteLength,
    a1_buf,
    a1_buf.byteLength,
    a2_buf,
    a2_buf.byteLength,
  );
  const result = rawResult.then(readPointer);
  return result.then(decode);
}
