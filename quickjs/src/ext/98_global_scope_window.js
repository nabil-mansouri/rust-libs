// Copyright 2018-2024 the Deno authors. All rights reserved. MIT license.

import { core, primordials } from "ext:core/mod.js";
/*[DEL]
import {
  op_bootstrap_language,
  op_bootstrap_numcpus,
  op_bootstrap_user_agent,
} from "ext:core/ops";
[DEL]*/
const {
  ObjectDefineProperties,
  ObjectPrototypeIsPrototypeOf,
  SymbolFor,
} = primordials;

import * as location from "ext:deno_web/12_location.js";
import * as console from "ext:deno_console/01_console.js";
import * as webidl from "ext:deno_webidl/00_webidl.js";
import * as globalInterfaces from "ext:deno_web/04_global_interfaces.js";
//[DEL] import * as webStorage from "ext:deno_webstorage/01_webstorage.js";
//[DEL] import * as prompt from "ext:runtime/41_prompt.js";
//[DEL] import { loadWebGPU } from "ext:deno_webgpu/00_init.js";

class Navigator {
  constructor() {
    webidl.illegalConstructor();
  }

  [SymbolFor("Deno.privateCustomInspect")](inspect, inspectOptions) {
    return inspect(
      console.createFilteredInspectProxy({
        object: this,
        evaluate: ObjectPrototypeIsPrototypeOf(NavigatorPrototype, this),
        keys: [
          "hardwareConcurrency",
          "userAgent",
          "language",
          "languages",
        ],
      }),
      inspectOptions,
    );
  }
}

const navigator = webidl.createBranded(Navigator);

function memoizeLazy(f) {
  let v_ = null;
  return () => {
    if (v_ === null) {
      v_ = f();
    }
    return v_;
  };
}

const numCpus = ()=> 1;//[DEL] memoizeLazy(() => op_bootstrap_numcpus());
const userAgent = ()=> "EN";//[DEL] memoizeLazy(() => op_bootstrap_user_agent());
const language = ()=> "EN";//[DEL] memoizeLazy(() => op_bootstrap_language());

ObjectDefineProperties(Navigator.prototype, {
  gpu: {
    configurable: true,
    enumerable: true,
    get() {
      webidl.assertBranded(this, NavigatorPrototype);
      //[DEL] const webgpu = loadWebGPU();
      //[DEL] return webgpu.gpu;
    },
  },
  hardwareConcurrency: {
    configurable: true,
    enumerable: true,
    get() {
      webidl.assertBranded(this, NavigatorPrototype);
      return numCpus();
    },
  },
  userAgent: {
    configurable: true,
    enumerable: true,
    get() {
      webidl.assertBranded(this, NavigatorPrototype);
      return userAgent();
    },
  },
  language: {
    configurable: true,
    enumerable: true,
    get() {
      webidl.assertBranded(this, NavigatorPrototype);
      return language();
    },
  },
  languages: {
    configurable: true,
    enumerable: true,
    get() {
      webidl.assertBranded(this, NavigatorPrototype);
      return [language()];
    },
  },
});
const NavigatorPrototype = Navigator.prototype;

const mainRuntimeGlobalProperties = {
  Location: location.locationConstructorDescriptor,
  location: location.locationDescriptor,
  Window: globalInterfaces.windowConstructorDescriptor,
  window: core.propGetterOnly(() => globalThis),
  self: core.propGetterOnly(() => globalThis),
  Navigator: core.propNonEnumerable(Navigator),
  navigator: core.propGetterOnly(() => navigator),
  alert:  core.propGetterOnly(()=>undefined),//[DEL] core.propWritable(prompt.alert),
  confirm:  core.propGetterOnly(()=>undefined),//[DEL] core.propWritable(prompt.confirm),
  prompt: core.propGetterOnly(()=>undefined),//[DEL] core.propWritable(prompt.prompt),
  localStorage: core.propGetterOnly(()=>undefined),//[DEL] core.propGetterOnly(webStorage.localStorage),
  sessionStorage: core.propGetterOnly(()=>undefined),//[DEL] core.propGetterOnly(webStorage.sessionStorage),
  Storage: core.propGetterOnly(()=>undefined),//[DEL] core.propNonEnumerable(webStorage.Storage),
};

export { mainRuntimeGlobalProperties, memoizeLazy };
