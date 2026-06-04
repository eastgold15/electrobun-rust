/**
 * Electrobun Macros
 *
 * 提供 Eden IPC 风格的宏，用于自动生成 FFI 绑定和 TypeScript 类型
 *
 * # 使用示例
 *
 * ```rust
 * use electrobun_macros::eden_ipc;
 *
 * // 定义 API trait
 * #[eden_ipc]
 * pub trait WindowAPI {
 *     fn create_window(&self, params: WindowParams) -> Result<Window, WindowError>;
 *     #[eden_stream]
 *     fn on_event(&self) -> impl Stream<Item = WindowEvent>;
 * }
 *
 * // 自动验证 impl 匹配 trait
 * #[eden_ipc(WindowAPI)]
 * impl ElectrobunApp {
 *     fn create_window(&self, params: WindowParams) -> Result<Window, WindowError> { ... }
 *     fn on_event(&self) -> impl Stream<Item = WindowEvent> { ... }
 * }
 * ```
 */
use proc_macro::TokenStream;
use syn::{parse_macro_input, ItemImpl, ItemTrait};

mod ffi_gen;
mod ipc;
mod stream;
mod ts_gen;

use ipc::{process_ipc_impl, process_ipc_trait};

/**
 * #[eden_ipc] 宏
 *
 * 用在 trait 上：标记 API，生成 FFI + TS
 * 用在 impl 上：验证签名匹配 trait，自动添加 impl X for Y
 */
#[proc_macro_attribute]
pub fn eden_ipc(attr: TokenStream, item: TokenStream) -> TokenStream {
    // 尝试解析为 Impl 块
    if let Ok(impl_block) = syn::parse::<ItemImpl>(item.clone()) {
        let trait_name_str = attr.to_string().trim().to_string();
        if trait_name_str.is_empty() {
            return syn::Error::new_spanned(&impl_block, "#[eden_ipc] on impl requires trait name, e.g. #[eden_ipc(WindowAPI)]")
                .to_compile_error().into();
        }
        match process_ipc_impl(impl_block, &trait_name_str) {
            Ok(output) => output.into(),
            Err(e) => e.to_compile_error().into(),
        }
    } else {
        // 否则按 trait 处理
        let input = parse_macro_input!(item as ItemTrait);
        match process_ipc_trait(input) {
            Ok(output) => output.into(),
            Err(e) => e.to_compile_error().into(),
        }
    }
}

/**
 * #[eden_stream] 宏
 *
 * 标记一个方法为流式方法，生成事件流支持。
 * 必须与 #[eden_ipc] 一起使用。
 */
#[proc_macro_attribute]
pub fn eden_stream(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
