use alloc::{format, string::*};
use core::ptr;
use rcstring::CString;

use crate::error::*;

#[inline]
pub fn as_cstring<V, T, F>(v: V, f: F) -> Result<T, Error>
where
    String: From<V>,
    F: FnOnce(CString<'_>) -> Result<T, Error>,
{
    let s: String = v.into();
    let string = format!("{}\0", s);
    f(CString::new(&string)?)
}

#[inline]
#[allow(dead_code)]
pub fn from_cstring(cstring: CString<'_>) -> String {
    unsafe { from_cstring_raw(cstring.into_raw()) }
}

#[inline]
pub unsafe fn from_cstring_raw(cstring: *const libc::c_char) -> String {
    let len = libc::strlen(cstring);
    let mut s = String::new();
    s.reserve(len);
    for _i in 0..len {
        s.push('\0');
    }
    ptr::copy(cstring, s.as_mut_ptr(), len);
    s
}
