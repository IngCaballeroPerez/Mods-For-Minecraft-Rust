use quote::quote;
use std::{collections::BTreeMap, env, fs, path::Path};
use syn::{Block, Item};
fn block<'a>(items: &'a mut [Item], node: &[usize]) -> &'a mut Block {
    let (index, rest) = node.split_first().unwrap();
    match &mut items[*index] {
        Item::Fn(f) if rest.is_empty() => &mut f.block,
        Item::Impl(i) if rest.len() == 1 => match &mut i.items[rest[0]] {
            syn::ImplItem::Fn(f) => &mut f.block,
            _ => panic!("planned method absent"),
        },
        Item::Mod(m) => block(&mut m.content.as_mut().unwrap().1, rest),
        _ => panic!("planned target absent"),
    }
}
fn canonical(items: &[Item]) -> String {
    prettyplease::unparse(&syn::File {
        shebang: None,
        attrs: vec![],
        items: items.to_vec(),
    })
}
// The printed form of a file once printing and parsing it again no longer
// changes it. One printing is not enough to compare two files: the printer
// may break a long line differently the second time, and the staged file has
// already been printed once by horizon-mixins.
fn settled(file: &syn::File) -> String {
    let mut text = prettyplease::unparse(file);
    for _ in 0..8 {
        let next = prettyplease::unparse(&syn::parse_file(&text).unwrap());
        if next == text {
            return text;
        }
        text = next;
    }
    panic!("the printed source does not settle");
}
fn main() {
    let args: Vec<_> = env::args().collect();
    let plan: serde_json::Value = serde_json::from_slice(&fs::read(&args[3]).unwrap()).unwrap();
    let mut trees = BTreeMap::new();
    for p in plan["injections"].as_array().unwrap() {
        let file = p["file"].as_str().unwrap();
        let tree = trees.entry(file).or_insert_with(|| {
            syn::parse_file(&fs::read_to_string(Path::new(&args[2]).join(file)).unwrap()).unwrap()
        });
        let node: Vec<usize> = p["node"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n.as_u64().unwrap() as usize)
            .collect();
        assert_eq!(p["kind"], "HEAD");
        assert_eq!(p["cancellable"], false);
        let helper: syn::ItemFn = syn::parse_str(p["helper_source"].as_str().unwrap()).unwrap();
        let name = &helper.sig.ident;
        let args: Vec<syn::Expr> = p["bindings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|b| syn::parse_str(b["value"].as_str().unwrap()).unwrap())
            .collect();
        let expected: syn::Stmt = syn::parse2(quote!({#helper #name(#(#args),*)};)).unwrap();
        let b = block(&mut tree.items, &node);
        let actual = b.stmts.first().expect("HEAD missing");
        let actual_fn: Item = syn::parse_quote!(fn audit() {#actual});
        let expected_fn: Item = syn::parse_quote!(fn audit() {#expected});
        assert_eq!(
            canonical(&[actual_fn]),
            canonical(&[expected_fn]),
            "HEAD differs from planned helper/bindings"
        );
        b.stmts.remove(0);
    }
    for m in plan["modules"].as_array().unwrap() {
        let file = m["file"].as_str().unwrap();
        // A module may go into a file no injection touches.
        let tree = trees.entry(file).or_insert_with(|| {
            syn::parse_file(&fs::read_to_string(Path::new(&args[2]).join(file)).unwrap()).unwrap()
        });
        let index = tree
            .items
            .iter()
            .position(|i| matches!(i,Item::Mod(v) if v.ident==m["name"].as_str().unwrap()))
            .expect("support module missing");
        let Item::Mod(module) = tree.items.remove(index) else {
            unreachable!()
        };
        let payload = syn::parse_file(m["source"].as_str().unwrap()).unwrap();
        assert!(module.attrs.is_empty());
        assert!(matches!(&module.vis, syn::Visibility::Restricted(v) if v.path.is_ident("crate")));
        let mut expected = module.clone();
        expected.content.as_mut().unwrap().1 = payload.items;
        assert!(
            canonical(&[Item::Mod(module)]) == canonical(&[Item::Mod(expected)]),
            "support module differs from planned source"
        );
    }
    for (file, tree) in trees {
        let original =
            syn::parse_file(&fs::read_to_string(Path::new(&args[1]).join(file)).unwrap()).unwrap();
        assert!(
            settled(&original) == settled(&tree),
            "unexpected source change in {file}"
        );
    }
    println!(
        "all original ASTs preserved apart from the planned HEAD statement and support module"
    );
}
