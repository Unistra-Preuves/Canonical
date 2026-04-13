use serde::{Deserialize, Serialize};
use std::{
    ffi::{c_char, CStr, CString},
    str::FromStr,
};

#[derive(Serialize, Deserialize, Debug)]
struct HSpine {
    shead: String,
    shargs: Vec<HTerm>,
}

#[derive(Serialize, Deserialize, Debug)]
struct HTerm {
    thead: Vec<String>,
    targs: HSpine,
}

#[no_mangle]
pub extern "C" fn hello(ptr: *const c_char) -> *mut c_char {
    unsafe {
        let s = CStr::from_ptr(ptr);
        println!("1: {}", s.to_str().unwrap());
        // let js: HTerm = serde_json::from_str(s.to_str().expect("Error 1\n")).expect("Error 2 \n");
        let t = HTerm {
            thead: Vec::from([String::from("x"), String::from("y")]),
            targs: HSpine {
                shead: String::from("x"),
                shargs: Vec::new(),
            },
        };

        let js = serde_json::to_string(&t).unwrap();
        println!("2: {}", js.as_str());

        let s2 = CString::from_str(js.as_str()).expect("Error 3\n");
        s2.into_raw()
        // let s: &'static CStr = CStr::from_ptr(s2.as_ptr());
        // let js2: HTerm = serde_json::from_str(s.to_str().expect("Error 1\n")).expect("Error 2 \n");
        // println!("{:?}", js2);
        // // s.as_ptr()
        // let t = CString::new("aaaa").unwrap();
        // t.into_raw()
    }
}
