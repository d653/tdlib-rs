use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("tdlib.rs");
    let mut outfile = File::create(&dest_path).unwrap();

    let f = File::open("td_api.tl").unwrap();
    let f = BufReader::new(&f);

    let mut ctors = vec![];
    let mut funcs = vec![];
    let mut istype = true;

    for l in f.lines() {
        let l = l.unwrap();

        if l == "---functions---" {
            istype = false;
        }
        if l == "---types---" {
            istype = true;
        }

        if let Some(parsed) = parse_fun(&l) {
            if istype { &mut ctors } else { &mut funcs }.push(parsed);
        }
    }

    let mut subtypes = HashMap::<String, Vec<String>>::new();
    for ctor in &ctors {
        subtypes
            .entry(ctor.result.clone())
            .or_insert(vec![])
            .push(ctor.name.clone());
    }

    let mut parents = HashMap::<String, String>::new();

    let mut abstracts = HashSet::<String>::new();

    for ctor in ctors.iter() {
        if subtypes[&ctor.result].len() > 1 {
            abstracts.insert(ctor.result.clone());
            parents.insert(ctor.name.clone(), ctor.result.clone());
        } else {
            parents.insert(ctor.name.clone(), "TLObject".into());
        }
    }
    for abs in &abstracts {
        parents.insert(abs.clone(), "TLObject".into());
    }
    for func in funcs.iter() {
        parents.insert(func.name.clone(), "TLFunction".into());
    }

    let natives = [
        ("Int32", "i32"),
        ("Int53", "i64"),
        ("Int64", "String"),
        ("Double", "f64"),
        ("Vector<T>", "Vec<T>"),
        ("string", "String"),
        ("Bytes", "String"),
        ("Bool", "bool"),
    ];

    let snatives: HashSet<String> = natives.iter().map(|&(x, _)| x.to_owned()).collect();

    writeln!(outfile, "use serde_derive::{{Deserialize, Serialize}};").unwrap();

    for (a, b) in &natives {
        writeln!(outfile, "type {} = {};", a, b).unwrap();
        if a.chars().next().unwrap().is_uppercase()
            && a.to_ascii_lowercase() != b.to_ascii_lowercase()
        {
            writeln!(outfile, "type {} = {};", lowercase_first_letter(a), a).unwrap();
        }
    }

    writeln!(outfile, "\n\n").unwrap();

    for (abs, subs) in subtypes.iter().filter(|(abs, _): &(&String, _)| {
        abstracts.contains(&abs.to_string()) && !snatives.contains(&abs.to_string())
    }) {
        writeln!(
            outfile,
            "#[derive(Serialize, Deserialize,Clone,Debug)]\n#[serde(untagged)]\npub enum {} {{",
            abs
        )
        .unwrap();
        for sub in subs {
            writeln!(
                outfile,
                "    {}({}),",
                &name_for_variant(sub, abs),
                uppercase_first_letter(sub)
            )
            .unwrap();
        }
        writeln!(outfile, "}}\n").unwrap();
    }

    for fun in ctors
        .iter()
        .chain(funcs.iter())
        .filter(|ctor| !snatives.contains(&ctor.name) && !snatives.contains(&parents[&ctor.name]))
    {
        let upper = uppercase_first_letter(&fun.name);
        writeln!(
            outfile,
            "#[derive(Serialize, Deserialize,Clone,Debug)]\n#[serde(deny_unknown_fields)]\n#[serde(tag = \"@type\")]").unwrap();
        writeln!(
            outfile,
            "pub enum E{} {{\n    #[serde(rename=\"{}\")]\n    {}\n}}",
            upper, fun.name, upper
        )
        .unwrap();
        writeln!(
            outfile,
            "#[derive(Serialize, Deserialize,Clone,Debug)]\npub struct {} {{",
            upper
        )
        .unwrap();
        writeln!(outfile, "    #[serde(flatten)]\n    tag : E{},", upper).unwrap();
        for (pn, pt) in &fun.params {
            let upt = uppercase_first_letter(pt);
            let (r, n) = rename_var(pn);
            if &parents[&fun.name] == pt {
                writeln!(outfile, "    pub {} : Box<{}>,", n, upt).unwrap();
            } else if r {
                writeln!(outfile, "    #[serde(rename = \"{}\")]", pn).unwrap();
                writeln!(outfile, "    pub {} : {},", n, upt).unwrap();
            } else {
                writeln!(outfile, "    pub {} : {},", pn, upt).unwrap();
            }
        }
        writeln!(outfile, "}}").unwrap();
        writeln!(outfile, "impl {} {{", upper).unwrap();
        let newparams = fun
            .params
            .iter()
            .map(|(pn, pt)| {
                let (_, pn) = rename_var(pn);
                let pt = uppercase_first_letter(pt);
                let t = if abstracts.contains(&pt) {
                    format!("impl Into<{}>", pt)
                } else {
                    pt
                };
                format!("{} : {}", pn, t)
            })
            .join(", ");
        writeln!(
            outfile,
            "    pub fn new ({}) -> Self {{\n        Self{{",
            newparams
        )
        .unwrap();
        let newvars = fun
            .params
            .iter()
            .map(|(pn, pt)| {
                let (_, pn) = rename_var(pn);
                if &parents[&fun.name] == pt {
                    format!("            {} : Box::new({}.into()),", pn, pn)
                } else if abstracts.contains(pt) {
                    format!("            {} : {}.into(),", pn, pn)
                } else {
                    format!("            {},", pn)
                }
            })
            .join("\n");
        writeln!(
            outfile,
            "            tag : E{}::{},\n{}",
            upper, upper, newvars
        )
        .unwrap();
        writeln!(outfile, "        }}\n    }}\n}}\n").unwrap();
    }

    for ty in ctors
        .iter()
        .filter(|ctor| !snatives.contains(&ctor.name) && !snatives.contains(&parents[&ctor.name]))
        .map(|ctor| ctor.name.to_owned())
        .chain(funcs.iter().map(|func| func.name.to_owned()))
        .chain(abstracts.iter().map(|x| x.to_owned()))
    {
        let upper = uppercase_first_letter(&ty);
        writeln!(
            outfile,
            "impl From<{}> for {} {{\n    fn from(x:{}) -> Self {{\n        {}::{}(x)\n    }}\n}}\n",
            upper,
            parents[&ty],
            upper,
            parents[&ty],
            name_for_variant(&ty, &parents[&ty])
        )
        .unwrap();
    }

    writeln!(
        outfile,
        "#[derive(Serialize, Deserialize,Clone,Debug)]\n#[serde(untagged)]\npub enum TLObject{{"
    )
    .unwrap();
    for ty in ctors
        .iter()
        .filter(|ctor| !snatives.contains(&ctor.name) && !snatives.contains(&parents[&ctor.name]))
        .map(|ctor| ctor.name.to_owned())
        .chain(funcs.iter().map(|func| func.name.to_owned()))
        .chain(abstracts.iter().map(|x| x.to_owned()))
    {
        if parents[&ty] == "TLObject" {
            let upper = uppercase_first_letter(&ty);
            writeln!(outfile, "    {}({}),", upper, upper).unwrap();
        }
    }
    writeln!(outfile, "}}\n").unwrap();

    writeln!(
        outfile,
        "#[derive(Serialize, Deserialize,Clone,Debug)]\n#[serde(untagged)]\npub enum TLFunction{{"
    )
    .unwrap();
    for ty in ctors
        .iter()
        .filter(|ctor| !snatives.contains(&ctor.name) && !snatives.contains(&parents[&ctor.name]))
        .map(|ctor| ctor.name.to_owned())
        .chain(funcs.iter().map(|func| func.name.to_owned()))
        .chain(abstracts.iter().map(|x| x.to_owned()))
    {
        if parents[&ty] == "TLFunction" {
            let upper = uppercase_first_letter(&ty);
            writeln!(outfile, "    {}({}),", upper, upper).unwrap();
        }
    }
    writeln!(outfile, "}}\n").unwrap();

    for fun in ctors.iter().chain(funcs.iter()) {
        let name = &fun.name;
        if !snatives.contains(name) && !snatives.contains(&parents[name]) {
            writeln!(
                outfile,
                "type {} = {};\n",
                name,
                uppercase_first_letter(name),
            )
            .unwrap();
        }
    }
}

fn uppercase_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}
fn lowercase_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
    }
}

fn name_for_variant(name: &str, abs: &str) -> String {
    let abslc = abs.to_ascii_lowercase().to_string();
    if name.to_ascii_lowercase().starts_with(&abslc) {
        name[abs.len()..].to_owned()
    } else {
        uppercase_first_letter(name)
    }
}

fn rename_var(s: &str) -> (bool, String) {
    if s != "type" {
        (false, s.to_owned())
    } else {
        (true, "type_".into())
    }
}

struct Function {
    name: String,
    params: Vec<(String, String)>,
    result: String,
}

fn parse_fun(l: &str) -> Option<Function> {
    if l.starts_with("//") {
        return None;
    }

    if !l.contains("=") {
        return None;
    }
    let mut v = l.split(";").next().unwrap().split("=");
    let fun = v.next().unwrap().trim().to_owned();
    let ret = v.next().unwrap().trim().to_owned();

    let mut f = fun.split(" ");
    let name = f.next().unwrap().into();
    let params = f
        .map(|p| {
            let mut it = p.split(":");
            let name = it.next().unwrap();
            let ty = it.next().unwrap();
            (name.into(), ty.into())
        })
        .collect();
    Some(Function {
        name,
        params,
        result: ret,
    })
}
