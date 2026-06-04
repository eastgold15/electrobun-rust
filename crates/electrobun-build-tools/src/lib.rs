//! Electrobun 工具集 — cdylib 入口
//!
//! 导出 C FFI 函数供 native wrapper C++ 加载使用：
//! - `asar_open` / `asar_read_file` / `asar_free_buffer` / `asar_close`

#![allow(clippy::expect_used)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::missing_errors_doc)]

pub mod asar;

/// 打开 ASAR 文件，返回不透明指针
#[no_mangle]
pub unsafe extern "C" fn asar_open(path: *const std::os::raw::c_char) -> *mut std::ffi::c_void {
    asar::asar_open(path)
}

/// 从 ASAR 中读取文件，成功返回 0，失败返回 -1
#[no_mangle]
pub unsafe extern "C" fn asar_read_file(
    archive: *mut std::ffi::c_void,
    filename: *const std::os::raw::c_char,
    out_buf: *mut *mut u8,
    out_len: *mut usize,
) -> i32 {
    asar::asar_read_file(archive, filename, out_buf, out_len)
}

/// 释放 asar_read_file 分配的缓冲区
#[no_mangle]
pub unsafe extern "C" fn asar_free_buffer(buf: *mut u8, len: usize) {
    asar::asar_free_buffer(buf, len);
}

/// 关闭 ASAR 归档
#[no_mangle]
pub unsafe extern "C" fn asar_close(archive: *mut std::ffi::c_void) {
    asar::asar_close(archive);
}
