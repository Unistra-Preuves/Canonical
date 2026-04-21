use canonical_compat::ir::*;
use canonical_core::core::*;
use canonical_core::memory::S;
use canonical_core::prover::*;
use canonical_core::search::*;
use serde::{Deserialize, Serialize};
use snailquote::unescape;
use std::ops::Deref;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
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
    lets: Vec<(String, HType)>,
    spine: HSpine,
}

fn to_ir_var(v: &String) -> IRVar {
    IRVar { name: v.to_owned() }
}

fn var_to_string(v: &IRVar) -> String {
    v.name.to_string()
}

fn to_ir_term(te: &HTerm) -> IRTerm {
    IRTerm {
        params: te.thead.iter().map(to_ir_var).collect(),
        lets: Vec::new(),
        spine: to_ir_spine(&te.targs),
        goal_rules: Vec::new(),
    }
}

fn to_hterm(te: &IRTerm) -> HTerm {
    HTerm {
        thead: te.params.iter().map(var_to_string).collect(),
        targs: to_hspine(&te.spine),
    }
}

fn to_ir_spine(sp: &HSpine) -> IRSpine {
    IRSpine {
        head: sp.shead.to_owned(),
        args: sp.sargs.iter().map(to_ir_term).collect(),
        premise_rules: Vec::new(),
    }
}

fn to_hspine(sp: &IRSpine) -> HSpine {
    HSpine {
        shead: sp.head.to_string(),
        sargs: sp.args.iter().map(to_hterm).collect(),
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

fn main(
    prover: Prover,
    sender: Sender<()>,
    count: usize,
    terms: Arc<Mutex<Vec<IRTerm>>>,
) -> (DFSResult, u32) {
    prover.prove(
        &|term: Term| {
            let mut v = terms.lock().unwrap();
            let bindings = term
                .base
                .borrow()
                .gamma
                .linked
                .as_ref()
                .unwrap()
                .borrow()
                .node
                .bindings
                .clone();
            let ir_term = IRTerm::from_lambda::<false>(term, bindings, false);
            if v.len() < count && v.iter().all(|x| x != &ir_term) {
                v.push(ir_term);
            }
            if v.len() >= count {
                RUN.store(false, Ordering::Relaxed);
                sender.send(()).unwrap();
            }
        },
        false,
    )
}

// static INSTANCE: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[no_mangle]
pub extern "C" fn canonical(
    typ: *const c_char,
    name: *const c_char,
    timeout: u64,
    count: usize,
) -> *mut c_char {
    unsafe {
        // get the type transmitted by haskell
        let cstr = CStr::from_ptr(typ);
        let rstr = unescape(cstr.to_str().unwrap()).unwrap();
        let typ: HType =
            serde_json::from_str(rstr.as_str()).expect("Failed to convert the JSON to a type.\n");

        // get the IRType
        let ir_type = to_ir_type(&typ);

        // Magic
        let (tx, rx) = mpsc::channel();

        let arc: Arc<Mutex<Vec<IRTerm>>> = Arc::new(Mutex::new(Vec::new()));
        let arc_clone = arc.clone();
        let tb = S::new(ir_type.to_type(&ES::new()));
        let problem_bind = S::new(Bind::new(
            CStr::from_ptr(name).to_string_lossy().into_owned(),
        ));
        let mut owned_linked = Vec::new();
        let prover = Prover::new(tb.downgrade(), problem_bind.downgrade(), &mut owned_linked);
        let worker = thread::spawn(move || main(prover, tx, count, arc_clone));
        let _ = rx.recv_timeout(Duration::from_secs(timeout));
        RUN.store(false, Ordering::Relaxed);
        let res: Vec<HTerm> = match worker.join() {
            Ok((_, _)) => {
                let v = arc.lock().unwrap();
                v.deref().iter().map(to_hterm).collect()
            }
            Err(e) => {
                let msg = if let Some(s) = e.downcast_ref::<String>() {
                    s.as_str()
                } else if let Some(s) = e.downcast_ref::<&'static str>() {
                    *s
                } else {
                    "internal panic"
                };
                panic!("{}", msg);
            }
        };

        // send back results to Haskell
        let json =
            serde_json::to_string(&res[0]).expect("Failed to convert type to a JSON format.\n");
        let cstr2 =
            CString::from_str(json.as_str()).expect("Failed to convert JSON to a C string.\n");
        cstr2.into_raw()
    }
}
