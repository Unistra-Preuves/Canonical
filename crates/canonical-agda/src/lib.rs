use canonical_compat::ir::*;
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
struct HType {
    bindings: Vec<(String, HType)>,
    spine: HSpine,
}

fn to_ir_var(v: &String) -> IRVar {
    IRVar { name: v.to_owned() }
}

fn to_ir_term(te: &HTerm) -> IRTerm {
    IRTerm {
        params: te.thead.iter().map(to_ir_var).collect(),
        lets: Vec::new(),
        spine: to_ir_spine(&te.targs),
        goal_rules: Vec::new(),
    }
}

fn to_ir_spine(sp: &HSpine) -> IRSpine {
    IRSpine {
        head: sp.shead.to_owned(),
        args: sp.sargs.iter().map(to_ir_term).collect(),
        premise_rules: Vec::new(),
    }
}

fn to_ir_type(ty: &HType) -> IRType {
    IRType {
        params: ty
            .bindings
            .iter()
            .map(|(_, t)| Some(to_ir_type(t.to_owned())))
            .collect(),
        lets: Vec::new(),
        codomain: IRTerm {
            params: ty
                .bindings
                .iter()
                .map(|(s, _)| IRVar { name: s.to_owned() })
                .collect(),
            lets: Vec::new(),
            spine: to_ir_spine(&ty.spine),
            goal_rules: Vec::new(),
        },
    }
}

#[no_mangle]
pub extern "C" fn canonical(ptr: *const c_char) -> *mut c_char {
    unsafe {
        let cstr = CStr::from_ptr(ptr);
        let rstr = unescape(cstr.to_str().unwrap()).unwrap();
        let typ: HType =
            serde_json::from_str(rstr.as_str()).expect("Failed to convert the JSON to a type.\n");
        let json = serde_json::to_string(&typ).expect("Failed to convert type to a JSON format.\n");
        let cstr2 =
            CString::from_str(json.as_str()).expect("Failed to convert JSON to a C string.\n");
        cstr2.into_raw()
    }
}
