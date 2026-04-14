use serde::{Deserialize, Serialize};
use snailquote::unescape;
use std::{
    ffi::{c_char, CStr, CString},
    str::FromStr,
};

#[derive(Serialize, Deserialize, Debug)]
struct HSpine {
    shead: String,
    sargs: Vec<HTerm>,
}

#[derive(Serialize, Deserialize, Debug)]
struct HTerm {
    thead: Vec<String>,
    targs: HSpine,
}

#[derive(Serialize, Deserialize, Debug)]
struct HTyp {
    bindings: Vec<(String, HTyp)>,
    spine: HSpine,
}

#[no_mangle]
pub extern "C" fn canonical(ptr: *const c_char) -> *mut c_char {
    unsafe {
        let cstr = CStr::from_ptr(ptr);
        let rstr = unescape(cstr.to_str().unwrap()).unwrap();
        let typ: HTyp =
            serde_json::from_str(rstr.as_str()).expect("Failed to convert the JSON to a type.\n");
        let json = serde_json::to_string(&typ).expect("Failed to convert type to a JSON format.\n");
        let cstr2 =
            CString::from_str(json.as_str()).expect("Failed to convert JSON to a C string.\n");
        cstr2.into_raw()
    }
}
