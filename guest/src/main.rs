#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::ToString as _;
use alloc::vec;
use alloc::vec::Vec;
use core::ffi::c_char;
use core::ops::Deref;
use core::ptr;

use hyperlight_common::flatbuffer_wrappers::function_call::FunctionCall;
use hyperlight_common::flatbuffer_wrappers::function_types::{
    ParameterType, ParameterValue, ReturnType,
};
use hyperlight_common::flatbuffer_wrappers::guest_error::ErrorCode;
use hyperlight_common::flatbuffer_wrappers::util::get_flatbuffer_result;
use hyperlight_guest::error::{HyperlightGuestError, Result};
use hyperlight_guest_bin::guest_function::definition::GuestFunctionDefinition;
use hyperlight_guest_bin::guest_function::register::register_function;
use mozjs_sys::glue::{DeleteCompileOptions, NewCompileOptions};
use mozjs_sys::jsapi::JS::{
    CompileGlobalScriptToStencil, DelazificationOption, Handle, InitSelfHostedCode,
    InstantiateGlobalStencil, InstantiateOptions, MutableHandle, OnNewGlobalHookOption,
    SelfHostedCache, SourceText, StencilRelease,
};
use mozjs_sys::jsapi::glue::{DeleteRealmOptions, JS_Init, JS_NewRealmOptions};
use mozjs_sys::jsapi::js::frontend::CompilationStencil;
use mozjs_sys::jsapi::mozilla::Utf8Unit;
use mozjs_sys::jsapi::*;
use mozjs_sys::jsval::UndefinedValue;

const DEFAULT_HEAP_MAX_BYTES: u32 = 16 * 1024 * 1024;

static mut ENGINE: Option<Engine> = None;

unsafe impl Sync for Engine {}

struct Engine {
    cx: *mut JSContext,
    global: *mut JSObject,
    stencil: Stencil,
}

fn engine() -> &'static mut Option<Engine> {
    // Safety: Oh, well...
    unsafe {
        let ptr: *mut Option<Engine> = &raw mut ENGINE;
        &mut *ptr
    }
}

impl Engine {
    fn with_script(source: &str) -> Result<Self> {
        unsafe {
            // Initialize JS engine
            if !JS_Init() {
                return Err(HyperlightGuestError::new(
                    ErrorCode::GuestError,
                    "Failed to initialize JS engine".to_string(),
                ));
            }

            let cx = JS_NewContext(DEFAULT_HEAP_MAX_BYTES, ptr::null_mut());
            if cx.is_null() {
                return Err(HyperlightGuestError::new(
                    ErrorCode::GuestError,
                    "Failed to create JS context".to_string(),
                ));
            }

            JS_SetGlobalJitCompilerOption(
                cx,
                JSJitCompilerOption::JSJITCOMPILER_PORTABLE_BASELINE_ENABLE,
                1,
            );
            JS_SetGlobalJitCompilerOption(
                cx,
                JSJitCompilerOption::JSJITCOMPILER_PORTABLE_BASELINE_WARMUP_THRESHOLD,
                0,
            );

            let realm_opts = JS_NewRealmOptions();
            if realm_opts.is_null() {
                return Err(HyperlightGuestError::new(
                    ErrorCode::GuestError,
                    "Failed to create realm options".to_string(),
                ));
            }

            let cache = SelfHostedCache::default();
            if !InitSelfHostedCode(cx, cache, None) {
                return Err(HyperlightGuestError::new(
                    ErrorCode::GuestError,
                    "Failed to initialize self-hosted code".to_string(),
                ));
            }

            let global = JS_NewGlobalObject(
                cx,
                &SIMPLE_GLOBAL_CLASS,
                ptr::null_mut(),
                OnNewGlobalHookOption::FireOnNewGlobalHook,
                realm_opts,
            );

            if global.is_null() {
                return Err(HyperlightGuestError::new(
                    ErrorCode::GuestError,
                    "Failed to create global object".to_string(),
                ));
            }

            let _ac = JSAutoRealm::new(cx, global);

            // Compile the script to stencil
            let filename = b"inline.js\0";
            let compile_opts = NewCompileOptions(cx, filename.as_ptr() as *const c_char, 1);
            if compile_opts.is_null() {
                return Err(HyperlightGuestError::new(
                    ErrorCode::GuestError,
                    "Failed to create compile options".to_string(),
                ));
            }

            (*compile_opts)._base.eagerDelazificationStrategy_ =
                DelazificationOption::ParseEverythingEagerly;

            let mut source_text = transform_str_to_source_text(source);
            let stencil_ref = CompileGlobalScriptToStencil(cx, compile_opts, &mut source_text);

            DeleteCompileOptions(compile_opts);
            DeleteRealmOptions(realm_opts);

            let stencil = Stencil { inner: stencil_ref };
            if stencil.is_null() {
                return Err(HyperlightGuestError::new(
                    ErrorCode::GuestError,
                    "Failed to compile script to stencil".to_string(),
                ));
            }

            Ok(Engine {
                cx,
                global,
                stencil,
            })
        }
    }

    fn execute(&self) -> Result<i32> {
        unsafe {
            let _ac = JSAutoRealm::new(self.cx, self.global);

            let instantiate_opts = InstantiateOptions {
                skipFilenameValidation: false,
                hideScriptFromDebugger: false,
                deferDebugMetadata: false,
            };

            let script = InstantiateGlobalStencil(
                self.cx,
                &instantiate_opts,
                *self.stencil,
                ptr::null_mut(),
            );

            if script.is_null() {
                return Err(HyperlightGuestError::new(
                    ErrorCode::GuestError,
                    "Failed to instantiate script from stencil".to_string(),
                ));
            }

            let mut rval = UndefinedValue();
            let rval_handle = MutableHandle::from_marked_location(&mut rval);

            let script_handle = Handle::from_marked_location(&script);
            let success = JS_ExecuteScript(self.cx, script_handle, rval_handle);

            if success && rval.is_int32() {
                Ok(rval.to_int32())
            } else {
                Err(HyperlightGuestError::new(
                    ErrorCode::GuestError,
                    "Script execution failed".to_string(),
                ))
            }
        }
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        // TODO(tandr): just port mozjs to no_std to do this for us
    }
}

pub fn init(function_call: &FunctionCall) -> Result<Vec<u8>> {
    let Some([ParameterValue::String(source)]) = function_call.parameters.as_deref() else {
        return Err(HyperlightGuestError::new(
            ErrorCode::GuestError,
            "Invalid parameters passed to Eval".to_string(),
        ));
    };

    let engine = engine();
    if engine.is_none() {
        *engine = Some(Engine::with_script(source)?);
    }

    Ok(get_flatbuffer_result(0))
}

pub fn exec() -> Result<Vec<u8>> {
    let engine = engine().as_mut().ok_or_else(|| {
        HyperlightGuestError::new(
            ErrorCode::GuestError,
            "Engine not initialized. Call Init first.".to_string(),
        )
    })?;

    let result = engine.execute()?;
    Ok(get_flatbuffer_result(result))
}

static SIMPLE_GLOBAL_CLASS_OPS: JSClassOps = JSClassOps {
    addProperty: None,
    delProperty: None,
    enumerate: Some(JS_EnumerateStandardClasses),
    newEnumerate: None,
    resolve: Some(JS_ResolveStandardClass),
    mayResolve: Some(JS_MayResolveStandardClass),
    finalize: None,
    call: None,
    construct: None,
    trace: Some(JS_GlobalObjectTraceHook),
};

static SIMPLE_GLOBAL_CLASS: JSClass = JSClass {
    name: b"Global\0".as_ptr() as *const c_char,
    flags: JSCLASS_IS_GLOBAL
        | ((JSCLASS_GLOBAL_SLOT_COUNT & JSCLASS_RESERVED_SLOTS_MASK)
            << JSCLASS_RESERVED_SLOTS_SHIFT),
    cOps: &SIMPLE_GLOBAL_CLASS_OPS as *const JSClassOps,
    spec: ptr::null(),
    ext: ptr::null(),
    oOps: ptr::null(),
};

pub struct Stencil {
    inner: already_AddRefed<CompilationStencil>,
}

impl Drop for Stencil {
    fn drop(&mut self) {
        if self.is_null() {
            return;
        }
        unsafe {
            StencilRelease(self.inner.mRawPtr);
        }
    }
}

impl Deref for Stencil {
    type Target = *mut CompilationStencil;

    fn deref(&self) -> &Self::Target {
        &self.inner.mRawPtr
    }
}

impl Stencil {
    pub fn is_null(&self) -> bool {
        self.inner.mRawPtr.is_null()
    }
}

fn transform_str_to_source_text(source: &str) -> SourceText<Utf8Unit> {
    SourceText {
        units_: source.as_ptr() as *const _,
        length_: source.len() as u32,
        ownsUnits_: false,
        _phantom_0: core::marker::PhantomData,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn hyperlight_main() {
    register_function(GuestFunctionDefinition::new(
        "Init".to_string(),
        vec![ParameterType::String],
        ReturnType::Int,
        init as usize,
    ));

    register_function(GuestFunctionDefinition::new(
        "Exec".to_string(),
        vec![],
        ReturnType::Int,
        exec as usize,
    ));
}

#[unsafe(no_mangle)]
pub fn guest_dispatch_function(function_call: FunctionCall) -> Result<Vec<u8>> {
    Err(HyperlightGuestError::new(
        ErrorCode::GuestFunctionNotFound,
        function_call.function_name.clone(),
    ))
}
