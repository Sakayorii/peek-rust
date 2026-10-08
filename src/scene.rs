//! The scene: draw() builds a plain node tree, backends render it.
//! Port of the node helpers in peek-vanilla's draw.js.

#[derive(Clone, Debug, PartialEq)]
pub struct SNode {
    pub tag: String,
    pub key: String,
    /// Insertion order is significant.
    pub attrs: Vec<(String, String)>,
    pub children: Vec<SNode>,
}

/// An attribute value before stringification, mirroring n()'s JS String().
#[derive(Clone, Debug)]
pub enum AttrVal {
    S(String),
    F(f64),
    I(i64),
    B(bool),
    None,
}

impl From<&str> for AttrVal {
    fn from(v: &str) -> Self {
        AttrVal::S(v.to_string())
    }
}
impl From<String> for AttrVal {
    fn from(v: String) -> Self {
        AttrVal::S(v)
    }
}
impl From<f64> for AttrVal {
    fn from(v: f64) -> Self {
        AttrVal::F(v)
    }
}
impl From<i32> for AttrVal {
    fn from(v: i32) -> Self {
        AttrVal::I(v as i64)
    }
}
impl From<i64> for AttrVal {
    fn from(v: i64) -> Self {
        AttrVal::I(v)
    }
}
impl From<u32> for AttrVal {
    fn from(v: u32) -> Self {
        AttrVal::I(v as i64)
    }
}
impl From<bool> for AttrVal {
    fn from(v: bool) -> Self {
        AttrVal::B(v)
    }
}
impl<T: Into<AttrVal>> From<Option<T>> for AttrVal {
    fn from(v: Option<T>) -> Self {
        match v {
            Some(x) => x.into(),
            None => AttrVal::None,
        }
    }
}

/// n(tag, key, attrs, children): values that are None or false are dropped,
/// everything else is stringified the way JS String() would.
pub fn n(tag: &str, key: &str, attrs: Vec<(&str, AttrVal)>, children: Vec<SNode>) -> SNode {
    let mut out = Vec::with_capacity(attrs.len());
    for (k, v) in attrs {
        let s = match v {
            AttrVal::None => continue,
            AttrVal::B(false) => continue,
            AttrVal::S(x) => x,
            AttrVal::F(x) => crate::js::js_num_to_string(x),
            AttrVal::I(x) => x.to_string(),
            AttrVal::B(true) => "true".to_string(),
        };
        out.push((k.to_string(), s));
    }
    SNode {
        tag: tag.to_string(),
        key: key.to_string(),
        attrs: out,
        children,
    }
}

/// Drops what cannot be seen: opacity 0, then groups left empty.
pub fn prune(node: &SNode) -> Option<SNode> {
    if node.attrs.iter().any(|(k, v)| k == "opacity" && v == "0") {
        return None;
    }
    let children: Vec<SNode> = node.children.iter().filter_map(prune).collect();
    if node.tag == "g" && children.is_empty() {
        return None;
    }
    let mut out = node.clone();
    out.children = children;
    Some(out)
}
