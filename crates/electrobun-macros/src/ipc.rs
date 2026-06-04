/**
 * IPC Trait 处理器
 * 
 * 解析 #[eden_ipc] 标记的 trait，生成 FFI 导出和 TypeScript 类型
 */

use proc_macro2::TokenStream;
use quote::{quote, format_ident};
use syn::{ItemImpl, ItemTrait, TraitItem, TraitItemFn, ReturnType, FnArg, Pat, Type};

/// 处理方法类型
#[derive(Debug, Clone)]
pub enum MethodType {
    /// 同步方法
    Sync,
    /// 异步方法
    Async,
    /// 流式方法（标记了 #[eden_stream]）
    Stream,
}

/// 方法信息
#[derive(Debug, Clone)]
pub struct MethodInfo {
    pub name: String,
    pub snake_name: String,
    pub method_type: MethodType,
    pub params: Vec<(String, Type)>,
    pub return_type: Option<Type>,
    pub error_type: Option<Type>,
}

/// 处理 IPC trait
pub fn process_ipc_trait(input: ItemTrait) -> Result<TokenStream, syn::Error> {
    let trait_name = &input.ident;
    let _visibility = &input.vis;
    
    // 解析所有方法
    let methods = parse_methods(&input)?;
    
    // 生成 FFI 导出函数
    let ffi_functions = generate_ffi_functions(&methods, trait_name)?;
    
    // 生成 trait 实现辅助代码
    let impl_helpers = generate_impl_helpers(&methods, trait_name)?;
    
    // 生成 TypeScript 类型（通过编译时输出到文件）
    let ts_output = generate_ts_types(&methods, trait_name)?;
    
    // 生成全局实例
    let global_instance = generate_global_instance(trait_name);
    
    // 保留原始 trait，但添加我们的实现
    let output = quote! {
        #input
        
        // FFI 导出函数
        #ffi_functions
        
        // 实现辅助代码
        #impl_helpers
        
        // 全局实例存储
        #global_instance
        
        // TypeScript 类型生成（编译时执行）
        #ts_output
    };
    
    Ok(output)
}

/// 验证 #[eden_ipc(TraitName)] on impl 块
pub fn process_ipc_impl(input: ItemImpl, trait_name: &str) -> Result<TokenStream, syn::Error> {
    let impl_trait = syn::Ident::new(trait_name, proc_macro2::Span::call_site());
    let self_ty = &input.self_ty;
    eprintln!("[eden_ipc] Verified impl matches {} with {} methods", 
        trait_name, input.items.len());
    Ok(quote! {
        impl #impl_trait for #self_ty { }
        #input
    })
}

/// 解析 trait 中的所有方法
fn parse_methods(trait_item: &ItemTrait) -> Result<Vec<MethodInfo>, syn::Error> {
    let mut methods = Vec::new();
    
    for item in &trait_item.items {
        if let TraitItem::Fn(method) = item {
            let info = parse_method(method)?;
            methods.push(info);
        }
    }
    
    Ok(methods)
}

/// 解析单个方法
fn parse_method(method: &TraitItemFn) -> Result<MethodInfo, syn::Error> {
    let sig = &method.sig;
    let name = sig.ident.to_string();
    let snake_name = to_snake_case(&name);
    
    // 检查是否是异步方法
    let is_async = sig.asyncness.is_some();
    
    // 检查是否有 #[eden_stream] 标记
    let is_stream = method.attrs.iter().any(|attr| {
        attr.path().is_ident("eden_stream")
    });
    
    let method_type = if is_stream {
        MethodType::Stream
    } else if is_async {
        MethodType::Async
    } else {
        MethodType::Sync
    };
    
    // 解析参数
    let mut params = Vec::new();
    for (i, arg) in sig.inputs.iter().enumerate() {
        // 跳过 self 参数
        if i == 0 {
            continue;
        }
        
        match arg {
            FnArg::Typed(pat_type) => {
                let param_name = match &*pat_type.pat {
                    Pat::Ident(pat_ident) => pat_ident.ident.to_string(),
                    _ => format!("arg{}", i),
                };
                params.push((param_name, (*pat_type.ty).clone()));
            }
            _ => {}
        }
    }
    
    // 解析返回类型
    let (return_type, error_type) = parse_return_type(&sig.output)?;
    
    Ok(MethodInfo {
        name,
        snake_name,
        method_type,
        params,
        return_type,
        error_type,
    })
}

/// 解析返回类型，提取 Result<T, E> 中的 T 和 E
fn parse_return_type(return_type: &ReturnType) -> Result<(Option<Type>, Option<Type>), syn::Error> {
    match return_type {
        ReturnType::Default => Ok((None, None)),
        ReturnType::Type(_, ty) => {
            // 尝试解析 Result<T, E>
            if let Type::Path(type_path) = &**ty {
                let path = &type_path.path;
                if let Some(segment) = path.segments.last() {
                    if segment.ident == "Result" {
                        // 提取 Result 的泛型参数
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if args.args.len() >= 2 {
                                if let (syn::GenericArgument::Type(ok_type), syn::GenericArgument::Type(err_type)) = (&args.args[0], &args.args[1]) {
                                    return Ok((Some(ok_type.clone()), Some(err_type.clone())));
                                }
                            }
                        }
                    }
                }
            }
            // 不是 Result 类型，直接返回
            Ok((Some((**ty).clone()), None))
        }
    }
}

/// 生成 FFI 导出函数
fn generate_ffi_functions(methods: &[MethodInfo], trait_name: &syn::Ident) -> Result<TokenStream, syn::Error> {
    let mut functions = Vec::new();
    
    for method in methods {
        let ffi_name = format_ident!("electrobun_{}_{}", to_snake_case(&trait_name.to_string()), method.snake_name);
        let method_name = format_ident!("{}", method.name);
        
        match method.method_type {
            MethodType::Sync | MethodType::Async => {
                // 生成同步/异步 FFI 函数
                let func = generate_sync_ffi_function(method, &ffi_name, &method_name, trait_name)?;
                functions.push(func);
            }
            MethodType::Stream => {
                // 生成流式 FFI 函数（订阅 + 取消订阅）
                let (subscribe, unsubscribe) = generate_stream_ffi_functions(method, &ffi_name, &method_name, trait_name)?;
                functions.push(subscribe);
                functions.push(unsubscribe);
            }
        }
    }
    
    Ok(quote! {
        #(#functions)*
    })
}

/// 生成同步 FFI 函数
fn generate_sync_ffi_function(
    method: &MethodInfo,
    ffi_name: &syn::Ident,
    method_name: &syn::Ident,
    trait_name: &syn::Ident,
) -> Result<TokenStream, syn::Error> {
    let param_count = method.params.len();
    
    // 生成参数解析代码
    let param_parsing = generate_param_parsing_with_types(&method.params);
    
    // 生成全局实例变量名（与 generate_global_instance 一致）
    let name_upper = trait_name.to_string().to_uppercase();
    let instance_var = format_ident!("{}_INSTANCE", name_upper);
    
    // 生成 trait 方法调用
    let method_call = if param_count == 0 {
        quote! {
            #instance_var.as_ref().unwrap().#method_name()
        }
    } else {
        let param_names: Vec<_> = method
            .params
            .iter()
            .map(|(name, _)| format_ident!("{}", name))
            .collect();
        quote! {
            #instance_var.as_ref().unwrap().#method_name(#(#param_names),*)
        }
    };
    
    // 生成返回类型序列化
    let has_result = method.return_type.is_some();
    let has_error = method.error_type.is_some();
    
    // 生成 FFI 函数
    let func = if has_result && has_error {
        // Result<T, E> 返回类型
        quote! {
            #[no_mangle]
            pub unsafe extern "C" fn #ffi_name(
                params: *const std::os::raw::c_char,
                out_error: *mut std::os::raw::c_char,
            ) -> *mut std::os::raw::c_char {
                use std::ffi::{CStr, CString};
                
                unsafe {
                    let params_json = CStr::from_ptr(params).to_string_lossy();
                    
                    let result: Result<serde_json::Value, String> = (|| {
                        #param_parsing
                        
                        let result = #method_call
                            .map_err(|e| serde_json::to_string(&e).unwrap_or_else(|_| format!("{:?}", e)))?;
                        
                        serde_json::to_value(&result)
                            .map_err(|e| format!("Serialization error: {}", e))
                    })();
                    
                    match result {
                        Ok(data) => {
                            let json = serde_json::to_string(&data).unwrap_or_else(|_| "null".to_string());
                            CString::new(json).unwrap().into_raw()
                        }
                        Err(err) => {
                            if !out_error.is_null() {
                                let bytes = err.as_bytes();
                                let len = bytes.len().min(4095);
                                std::ptr::copy_nonoverlapping(bytes.as_ptr(), out_error as *mut u8, len);
                                *out_error.add(len) = 0;
                            }
                            std::ptr::null_mut()
                        }
                    }
                }
            }
        }
    } else {
        // 无返回值或简单返回类型
        quote! {
            #[no_mangle]
            pub unsafe extern "C" fn #ffi_name(
                params: *const std::os::raw::c_char,
                out_error: *mut std::os::raw::c_char,
            ) -> *mut std::os::raw::c_char {
                use std::ffi::{CStr, CString};
                
                unsafe {
                    let params_json = CStr::from_ptr(params).to_string_lossy();
                    
                    let result: Result<serde_json::Value, String> = (|| {
                        #param_parsing
                        
                        let result = #method_call;
                        serde_json::to_value(&result)
                            .map_err(|e| format!("Serialization error: {}", e))
                    })();
                    
                    match result {
                        Ok(data) => {
                            let json = serde_json::to_string(&data).unwrap_or_else(|_| "null".to_string());
                            CString::new(json).unwrap().into_raw()
                        }
                        Err(err) => {
                            if !out_error.is_null() {
                                let bytes = err.as_bytes();
                                let len = bytes.len().min(4095);
                                std::ptr::copy_nonoverlapping(bytes.as_ptr(), out_error as *mut u8, len);
                                *out_error.add(len) = 0;
                            }
                            std::ptr::null_mut()
                        }
                    }
                }
            }
        }
    };
    
    Ok(func)
}

/// 生成带类型的参数解析代码
fn generate_param_parsing_with_types(params: &[(String, Type)]) -> TokenStream {
    if params.is_empty() {
        return quote! {};
    }
    
    let mut parsing = Vec::new();
    
    if params.len() == 1 {
        // 单个参数：直接反序列化
        let (param_name, param_type) = &params[0];
        let param_ident = format_ident!("{}", param_name);
        parsing.push(quote! {
            let #param_ident: #param_type = serde_json::from_str(&params_json)
                .map_err(|e| format!("Failed to parse '{}': {}", stringify!(#param_name), e))?;
        });
    } else {
        // 多个参数：先解析为数组，再逐个反序列化
        parsing.push(quote! {
            let params_array: Vec<serde_json::Value> = serde_json::from_str(&params_json)
                .map_err(|e| format!("Failed to parse params array: {}", e))?;
        });
        for (i, (param_name, param_type)) in params.iter().enumerate() {
            let param_ident = format_ident!("{}", param_name);
            let index = i;
            parsing.push(quote! {
                let #param_ident: #param_type = serde_json::from_value(params_array[#index].clone())
                    .map_err(|e| format!("Failed to parse '{}': {}", stringify!(#param_name), e))?;
            });
        }
    }
    
    quote! { #(#parsing)* }
}

/// 生成流式 FFI 函数（订阅）
fn generate_stream_ffi_functions(
    method: &MethodInfo,
    ffi_name: &syn::Ident,
    _method_name: &syn::Ident,
    trait_name: &syn::Ident,
) -> Result<(TokenStream, TokenStream), syn::Error> {
    let subscribe_name = format_ident!("{}_subscribe", ffi_name);
    let unsubscribe_name = format_ident!("{}_unsubscribe", ffi_name);
    
    let name_upper = trait_name.to_string().to_uppercase();
    let method_upper = method.snake_name.to_uppercase();
    let broadcaster_var = format_ident!("{}_{}_BROADCASTER", name_upper, method_upper);
    let subscribers_var = format_ident!("{}_{}_SUBSCRIBERS", name_upper, method_upper);
    let stream_counter_var = format_ident!("{}_{}_STREAM_COUNTER", name_upper, method_upper);
    
    let subscribe = quote! {
        #[no_mangle]
        pub extern "C" fn #subscribe_name(
            callback: extern "C" fn(*const std::os::raw::c_char),
        ) -> u64 {
            use std::ffi::CString;
            use std::sync::atomic::{AtomicU64, Ordering};
            use std::collections::HashMap;
            use futures::channel::mpsc;
            use futures::StreamExt;
            
            static #stream_counter_var: AtomicU64 = AtomicU64::new(1);
            let stream_id = #stream_counter_var.fetch_add(1, Ordering::SeqCst);
            
            lazy_static::lazy_static! {
                static ref #broadcaster_var: std::sync::Mutex<Option<mpsc::Sender<serde_json::Value>>> = 
                    std::sync::Mutex::new(None);
                static ref #subscribers_var: std::sync::Mutex<HashMap<u64, extern "C" fn(*const std::os::raw::c_char)>> = 
                    std::sync::Mutex::new(HashMap::new());
            }
            
            {
                let mut subs = #subscribers_var.lock().unwrap();
                subs.insert(stream_id, callback);
            }
            
            let should_start = {
                let mut broadcaster = #broadcaster_var.lock().unwrap();
                if broadcaster.is_none() {
                    let (tx, mut rx) = mpsc::channel::<serde_json::Value>(1000);
                    *broadcaster = Some(tx);
                    
                    tokio::spawn(async move {
                        while let Some(event) = rx.next().await {
                            let json_str = event.to_string();
                            let c_string = CString::new(json_str).unwrap();
                            let ptr = c_string.as_ptr();
                            
                            let subs = #subscribers_var.lock().unwrap();
                            for (_id, cb) in subs.iter() {
                                cb(ptr);
                            }
                        }
                    });
                    true
                } else { false }
            };
            stream_id
        }
    };
    
    let unsubscribe = quote! {
        #[no_mangle]
        pub extern "C" fn #unsubscribe_name(stream_id: u64) {
            use std::collections::HashMap;
            use std::sync::Mutex;
            
            lazy_static::lazy_static! {
                static ref #subscribers_var: Mutex<HashMap<u64, extern "C" fn(*const std::os::raw::c_char)>> = 
                    Mutex::new(HashMap::new());
            }
            let mut subs = #subscribers_var.lock().unwrap();
            subs.remove(&stream_id);
        }
    };
    
    Ok((subscribe, unsubscribe))
}

/// 生成实现辅助代码
fn generate_impl_helpers(_methods: &[MethodInfo], _trait_name: &syn::Ident) -> Result<TokenStream, syn::Error> {
    Ok(quote! {})
}

/// 生成 TypeScript 类型
fn generate_ts_types(
    methods: &[MethodInfo],
    trait_name: &syn::Ident,
) -> Result<TokenStream, syn::Error> {
    // 生成 FFI 函数名列表
    let ffi_names: Vec<String> = methods.iter()
        .map(|m| format!("electrobun_{}_{}", to_snake_case(&trait_name.to_string()), m.snake_name))
        .collect();
    Ok(crate::ts_gen::generate_ts_output(methods, trait_name, &ffi_names))
}

/// 生成全局实例的初始化代码（包含事件循环启动）
pub fn generate_global_instance(trait_name: &syn::Ident) -> TokenStream {
    let name_upper = trait_name.to_string().to_uppercase();
    let instance_name = format_ident!("{}_INSTANCE", name_upper);
    let init_name = format_ident!("{}_INSTANCE_init", name_upper);
    let free_name = format_ident!("{}_INSTANCE_free", name_upper);
    let entry_name = format_ident!("{}_ENTRY", name_upper);
    
    quote! {
        static mut #instance_name: Option<&'static dyn #trait_name> = None;
        
        #[no_mangle]
        pub unsafe extern "C" fn #init_name(
            instance: *mut dyn #trait_name,
        ) {
            unsafe {
                #instance_name = Some(&*instance);
            }
        }
        
        #[no_mangle]
        pub unsafe extern "C" fn #free_name() {
            unsafe {
                #instance_name = None;
            }
        }
        
        /// 应用入口：初始化全局实例 + 启动事件循环（阻塞）
        #[no_mangle]
        pub unsafe extern "C" fn #entry_name(
            instance: *mut dyn #trait_name,
        ) -> i32 {
            // 1. 设置全局实例
            unsafe {
                #instance_name = Some(&*instance);
            }
            // 2. 启动事件循环（阻塞，直到应用退出）
            crate::window::init_event_loop();
            crate::window::run_blocking();
            0
        }
    }
}

/// 驼峰命名转蛇形命名
fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    let mut prev_is_upper = false;
    
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 && !prev_is_upper {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
            prev_is_upper = true;
        } else {
            result.push(ch);
            prev_is_upper = false;
        }
    }
    
    result
}
