// Copyright 2018-2024 the Deno authors. All rights reserved. MIT license.

import { core } from "ext:core/mod.js";

import * as event from "ext:deno_web/02_event.js";
import * as timers from "ext:deno_web/02_timers.js";
import * as base64 from "ext:deno_web/05_base64.js";
import * as encoding from "ext:deno_web/08_text_encoding.js";
import * as console from "ext:deno_console/01_console.js";
//[DEL] import * as caches from "ext:deno_cache/01_cache.js";
import * as compression from "ext:deno_web/14_compression.js";
//[DEL] import * as worker from "ext:runtime/11_workers.js";
import * as performance from "ext:deno_web/15_performance.js";
//[DEL] import * as crypto from "ext:deno_crypto/00_crypto.js";
import * as url from "ext:deno_url/00_url.js";
import * as urlPattern from "ext:deno_url/01_urlpattern.js";
import * as headers from "ext:deno_fetch/20_headers.js";
import * as streams from "ext:deno_web/06_streams.js";
import * as fileReader from "ext:deno_web/10_filereader.js";
//[DEL] import * as webSocket from "ext:deno_websocket/01_websocket.js";
//[DEL] import * as webSocketStream from "ext:deno_websocket/02_websocketstream.js";
//[DEL] import * as broadcastChannel from "ext:deno_broadcast_channel/01_broadcast_channel.js";
import * as file from "ext:deno_web/09_file.js";
import * as formData from "ext:deno_fetch/21_formdata.js";
import * as request from "ext:deno_fetch/23_request.js";
import * as response from "ext:deno_fetch/23_response.js";
import * as fetch from "ext:deno_fetch/26_fetch.js";
import * as eventSource from "ext:deno_fetch/27_eventsource.js";
import * as messagePort from "ext:deno_web/13_message_port.js";
import * as webidl from "ext:deno_webidl/00_webidl.js";
import { DOMException } from "ext:deno_web/01_dom_exception.js";
import * as abortSignal from "ext:deno_web/03_abort_signal.js";
import * as imageData from "ext:deno_web/16_image_data.js";
//[DEL] import { loadWebGPU } from "ext:deno_webgpu/00_init.js";
//[DEL] import * as webgpuSurface from "ext:deno_webgpu/02_surface.js";
//[DEL] import { unstableIds } from "ext:runtime/90_deno_ns.js";

const loadImage = core.createLazyLoader("ext:deno_canvas/01_image.js");

// https://developer.mozilla.org/en-US/docs/Web/API/WindowOrWorkerGlobalScope
const windowOrWorkerGlobalScope = {
  AbortController: core.propNonEnumerable(abortSignal.AbortController),
  AbortSignal: core.propNonEnumerable(abortSignal.AbortSignal),
  Blob: core.propNonEnumerable(file.Blob),
  ByteLengthQueuingStrategy: core.propNonEnumerable(
    streams.ByteLengthQueuingStrategy,
  ),
  CloseEvent: core.propNonEnumerable(event.CloseEvent),
  CompressionStream: core.propNonEnumerable(compression.CompressionStream),
  CountQueuingStrategy: core.propNonEnumerable(
    streams.CountQueuingStrategy,
  ),
  CryptoKey: core.propGetterOnly(()=>undefined), //[DEL] core.propNonEnumerable(crypto.CryptoKey),
  CustomEvent: core.propNonEnumerable(event.CustomEvent),
  DecompressionStream: core.propNonEnumerable(compression.DecompressionStream),
  DOMException: core.propNonEnumerable(DOMException),
  ErrorEvent: core.propNonEnumerable(event.ErrorEvent),
  Event: core.propNonEnumerable(event.Event),
  EventTarget: core.propNonEnumerable(event.EventTarget),
  File: core.propNonEnumerable(file.File),
  FileReader: core.propNonEnumerable(fileReader.FileReader),
  FormData: core.propNonEnumerable(formData.FormData),
  Headers: core.propNonEnumerable(headers.Headers),
  ImageData: core.propNonEnumerable(imageData.ImageData),
  ImageBitmap: core.propNonEnumerableLazyLoaded(
    (image) => image.ImageBitmap,
    loadImage,
  ),
  MessageEvent: core.propNonEnumerable(event.MessageEvent),
  Performance: core.propNonEnumerable(performance.Performance),
  PerformanceEntry: core.propNonEnumerable(performance.PerformanceEntry),
  PerformanceMark: core.propNonEnumerable(performance.PerformanceMark),
  PerformanceMeasure: core.propNonEnumerable(performance.PerformanceMeasure),
  PromiseRejectionEvent: core.propNonEnumerable(event.PromiseRejectionEvent),
  ProgressEvent: core.propNonEnumerable(event.ProgressEvent),
  ReadableStream: core.propNonEnumerable(streams.ReadableStream),
  ReadableStreamDefaultReader: core.propNonEnumerable(
    streams.ReadableStreamDefaultReader,
  ),
  Request: core.propNonEnumerable(request.Request),
  Response: core.propNonEnumerable(response.Response),
  TextDecoder: core.propNonEnumerable(encoding.TextDecoder),
  TextEncoder: core.propNonEnumerable(encoding.TextEncoder),
  TextDecoderStream: core.propNonEnumerable(encoding.TextDecoderStream),
  TextEncoderStream: core.propNonEnumerable(encoding.TextEncoderStream),
  TransformStream: core.propNonEnumerable(streams.TransformStream),
  URL: core.propNonEnumerable(url.URL),
  URLPattern: core.propNonEnumerable(urlPattern.URLPattern),
  URLSearchParams: core.propNonEnumerable(url.URLSearchParams),
  WebSocket: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerable(webSocket.WebSocket),
  MessageChannel: core.propNonEnumerable(messagePort.MessageChannel),
  MessagePort: core.propNonEnumerable(messagePort.MessagePort),
  Worker: core.propGetterOnly(()=>undefined), //[DEL] core.propNonEnumerable(worker.Worker),
  WritableStream: core.propNonEnumerable(streams.WritableStream),
  WritableStreamDefaultWriter: core.propNonEnumerable(
    streams.WritableStreamDefaultWriter,
  ),
  WritableStreamDefaultController: core.propNonEnumerable(
    streams.WritableStreamDefaultController,
  ),
  ReadableByteStreamController: core.propNonEnumerable(
    streams.ReadableByteStreamController,
  ),
  ReadableStreamBYOBReader: core.propNonEnumerable(
    streams.ReadableStreamBYOBReader,
  ),
  ReadableStreamBYOBRequest: core.propNonEnumerable(
    streams.ReadableStreamBYOBRequest,
  ),
  ReadableStreamDefaultController: core.propNonEnumerable(
    streams.ReadableStreamDefaultController,
  ),
  TransformStreamDefaultController: core.propNonEnumerable(
    streams.TransformStreamDefaultController,
  ),
  atob: core.propWritable(base64.atob),
  btoa: core.propWritable(base64.btoa),
  createImageBitmap: core.propWritableLazyLoaded(
    (image) => image.createImageBitmap,
    loadImage,
  ),
  clearInterval: core.propWritable(timers.clearInterval),
  clearTimeout: core.propWritable(timers.clearTimeout),
  caches: {
    enumerable: true,
    configurable: true,
    get: ()=>undefined, //[DEL] caches.cacheStorage,
  },
  CacheStorage: core.propGetterOnly(()=>undefined), //[DEL] core.propNonEnumerable(caches.CacheStorage),
  Cache: core.propGetterOnly(()=>undefined), //[DEL] core.propNonEnumerable(caches.Cache),
  console: core.propNonEnumerable(
    new console.Console((msg, level) => core.print(msg, level > 1)),
  ),
  crypto: core.propGetterOnly(()=>undefined),//[DEL] core.propReadOnly(crypto.crypto),
  Crypto: core.propGetterOnly(()=>undefined), //[DEL] core.propNonEnumerable(crypto.Crypto),
  SubtleCrypto: core.propGetterOnly(()=>undefined), //[DEL] core.propNonEnumerable(crypto.SubtleCrypto),
  fetch: core.propWritable(fetch.fetch),
  EventSource: core.propWritable(eventSource.EventSource),
  performance: core.propWritable(performance.performance),
  reportError: core.propWritable(event.reportError),
  setInterval: core.propWritable(timers.setInterval),
  setTimeout: core.propWritable(timers.setTimeout),
  structuredClone: core.propWritable(messagePort.structuredClone),
  // Branding as a WebIDL object
  [webidl.brand]: core.propNonEnumerable(webidl.brand),
};

const unstableForWindowOrWorkerGlobalScope = { __proto__: null };
/*[DEL] unstableForWindowOrWorkerGlobalScope[unstableIds.broadcastChannel] = {
  BroadcastChannel: core.propGetterOnly(()=>undefined), //[DEL] core.propNonEnumerable(broadcastChannel.BroadcastChannel),
};[DEL]*/
/*[DEL] unstableForWindowOrWorkerGlobalScope[unstableIds.net] = {
  WebSocketStream: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerable(webSocketStream.WebSocketStream),
  WebSocketError: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerable(webSocketStream.WebSocketError),
};[DEL]*/
// deno-fmt-ignore
/*[DEL] 
unstableForWindowOrWorkerGlobalScope[unstableIds.webgpu] = {
  GPU: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPU, loadWebGPU),
  GPUAdapter: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPUAdapter, loadWebGPU),
  GPUAdapterInfo: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPUAdapterInfo, loadWebGPU),
  GPUSupportedLimits: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPUSupportedLimits, loadWebGPU),
  GPUSupportedFeatures: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPUSupportedFeatures, loadWebGPU),
  GPUDeviceLostInfo: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPUDeviceLostInfo, loadWebGPU),
  GPUDevice: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPUDevice, loadWebGPU),
  GPUQueue: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPUQueue, loadWebGPU),
  GPUBuffer: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPUBuffer, loadWebGPU),
  GPUBufferUsage: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPUBufferUsage, loadWebGPU),
  GPUMapMode: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPUMapMode, loadWebGPU),
  GPUTextureUsage: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPUTextureUsage, loadWebGPU),
  GPUTexture: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPUTexture, loadWebGPU),
  GPUTextureView: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPUTextureView, loadWebGPU),
  GPUSampler: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPUSampler, loadWebGPU),
  GPUBindGroupLayout: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPUBindGroupLayout, loadWebGPU),
  GPUPipelineLayout: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPUPipelineLayout, loadWebGPU),
  GPUBindGroup: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPUBindGroup, loadWebGPU),
  GPUShaderModule: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPUShaderModule, loadWebGPU),
  GPUShaderStage: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPUShaderStage, loadWebGPU),
  GPUComputePipeline: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPUComputePipeline, loadWebGPU),
  GPURenderPipeline: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPURenderPipeline, loadWebGPU),
  GPUColorWrite: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPUColorWrite, loadWebGPU),
  GPUCommandEncoder: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPUCommandEncoder, loadWebGPU),
  GPURenderPassEncoder: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPURenderPassEncoder, loadWebGPU),
  GPUComputePassEncoder: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPUComputePassEncoder, loadWebGPU),
  GPUCommandBuffer: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPUCommandBuffer, loadWebGPU),
  GPURenderBundleEncoder: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPURenderBundleEncoder, loadWebGPU),
  GPURenderBundle: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPURenderBundle, loadWebGPU),
  GPUQuerySet: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPUQuerySet, loadWebGPU),
  GPUError: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPUError, loadWebGPU),
  GPUValidationError: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPUValidationError, loadWebGPU),
  GPUOutOfMemoryError: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerableLazyLoaded((webgpu) => webgpu.GPUOutOfMemoryError, loadWebGPU),
  GPUCanvasContext: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerable(webgpuSurface.GPUCanvasContext),
};
[DEL]*/
export { unstableForWindowOrWorkerGlobalScope, windowOrWorkerGlobalScope };
