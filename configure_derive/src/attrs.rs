use syn::*;

pub struct CfgAttrs {
    pub name: Option<String>,
    pub docs: bool,
}

impl CfgAttrs {
    pub fn new(attrs: &[Attribute]) -> CfgAttrs {
        let cfg_attrs = filter_attrs(attrs);

        let mut cfg = CfgAttrs {
            name: None,
            docs: false,
        };

        // Parse the cfg attrs
        for attr in cfg_attrs {
            if let NestedMetaItem::MetaItem(ref attr) = *attr {
                match attr.name() {
                    "name" if cfg.name.is_some()    => panic!("Multiple `name` attributes"),
                    "name"                          => cfg.name = project_name(attr),
                    "generate_docs" if cfg.docs     => panic!("Multiple `generate_docs` attributes"),
                    "generate_docs"                 => cfg.docs = gen_docs(attr),
                    unknown                         => {
                        panic!("Unrecognized configure attribute `{}`", unknown)
                    }
                }
            } else { panic!("Unrecognized configure attribute literal") }
        }

        cfg
    }
}

pub struct FieldAttrs {
    pub docs: Option<String>,
}

impl FieldAttrs {
    pub fn new(field: &Field) -> FieldAttrs {
        let cfg_attrs = filter_attrs(&field.attrs);

        let mut cfg = FieldAttrs { docs: None };

        for attr in cfg_attrs {
            if let NestedMetaItem::MetaItem(ref attr) = *attr {
                match attr.name() {
                    "docs" if cfg.docs.is_some()    => {
                        let name = field.ident.as_ref().unwrap();
                        panic!("Multiple `docs` attributes on one field: `{}`.", name)
                    }
                    "docs"                          => {
                        cfg.docs = field_docs(attr)
                    }
                    unknown                         => {
                        panic!("Unrecognized configure attribute `{}`", unknown)
                    }
                }
            } else { panic!("Unrecognized configure attribute literal") }
        }

        cfg
    }
}

fn filter_attrs(attrs: &[Attribute]) -> Vec<&NestedMetaItem> {
    let mut cfg_attrs = vec![];
    for attr in attrs {
        match attr.value {
            MetaItem::List(ref name, ref members) if name.as_ref() == "configure"   => {
                cfg_attrs.extend(members);
            }
            _   => continue
        }
    }

    cfg_attrs
}

fn project_name(attr: &MetaItem) -> Option<String> {
    if let MetaItem::NameValue(_, ref name) = *attr {
        if let Lit::Str(ref string, _) = *name {
            return Some(string.clone())
        }
    }
    panic!("Unsupported `configure(name)` attribute; only supported form is #[configure(name = \"$NAME\")]")
}

fn gen_docs(attr: &MetaItem) -> bool {
    if let MetaItem::Word(_) = *attr {
        return true
    } else  {
        panic!("Unsupported `configure(docs)` attribute; only supported form is #[configure(docs)]")
    }
}

fn field_docs(attr: &MetaItem) -> Option<String> {
    if let MetaItem::NameValue(_, ref name) = *attr {
        if let Lit::Str(ref string, _) = *name {
            return Some(string.clone())
        }
    }
    panic!("Unsupported `configure(docs)` attribute; only supported form is #[configure(docs = \"$NAME\")]")
}
