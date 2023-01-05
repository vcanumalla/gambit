use itertools::Itertools;
use regex::Regex;
use serde::Deserialize;
use serde_json::Value;

/// This is a thin wrapper around the json AST
/// generated by the solidity compiler.

#[derive(Debug, Deserialize, Default, Clone)]
pub struct TypeDescriptions {
    pub(crate) element: Option<Value>,
}

impl TypeDescriptions {
    pub fn new(v: Value) -> Self {
        if v.is_null() {
            Self { element: None }
        } else {
            Self { element: Some(v) }
        }
    }

    pub fn type_string(&self) -> Option<String> {
        self.element.as_ref().map(|e| e["typeString"].to_string())
    }
}

struct Replacement {
    start: usize,
    end: usize,
    new: String,
}

/// Solidity AST representation.
/// There are two fields, `element`
/// which is the underlying json object representing
/// an AST node and `contract` which indicates
/// the name of the contract that this node belongs to.
#[derive(Debug, Deserialize, Default, Clone)]
#[serde(default)]
pub struct SolAST {
    pub(crate) element: Option<Value>,
    pub(crate) contract: Option<String>,
}

impl SolAST {
    /// Create a new AST node.
    pub fn new(v: Value, c: Option<String>) -> Self {
        if v.is_null() {
            Self {
                element: None,
                contract: None,
            }
        } else {
            Self {
                element: Some(v),
                contract: c,
            }
        }
    }

    /// Return the `element` field of a `SolAST` struct.
    pub fn get_object(&self) -> Option<Value> {
        self.element.clone()
    }

    /// Return the `contract` field of a `SolAST` struct.
    pub fn get_contract(&self) -> Option<String> {
        self.contract.clone()
    }

    /// Return some node of this AST that has the field name `fnm` in the json
    /// representation.
    pub fn get_node(&self, fnm: &str) -> SolAST {
        let node: SolAST = self.get_object().map_or_else(
            || SolAST {
                element: None,
                contract: self.get_contract(),
            },
            |v| SolAST {
                element: Some(v[fnm].clone()),
                contract: self.get_contract(),
            },
        );
        node
    }

    /// A helper that is used in various places to get the value of some
    /// field name (`fnm`) in the AST's `element`.
    pub fn get_string(&self, fnm: &str) -> Option<String> {
        let obj = self.get_object();
        match obj {
            Some(o) => {
                let v = o[fnm].as_str();
                v.map(|s| s.into())
            }
            None => None,
        }
    }

    /// Returns the `src` field.
    pub fn src(&self) -> Option<String> {
        self.get_string("src")
    }

    /// Returns the `name` field.
    pub fn name(&self) -> Option<String> {
        self.get_string("name")
    }

    /// Returns the `node_type` field.
    pub fn node_type(&self) -> Option<String> {
        self.get_string("nodeType")
    }

    /// Returns the `expression` field.
    pub fn expression(&self) -> SolAST {
        self.get_node("expression")
    }

    /// Returns the `operator` field.
    pub fn operator(&self) -> Option<String> {
        self.get_string("operator")
    }

    /// Returns the `leftExpression` field.
    pub fn left_expression(&self) -> SolAST {
        self.get_node("leftExpression")
    }

    /// Returns the `rightExpression` field.
    pub fn right_expression(&self) -> SolAST {
        self.get_node("rightExpression")
    }

    /// Returns the `leftHandSide` field.
    pub fn left_hand_side(&self) -> SolAST {
        self.get_node("leftHandSide")
    }

    /// Returns the `rightHandSide` field.
    pub fn right_hand_side(&self) -> SolAST {
        self.get_node("rightHandSide")
    }

    /// Returns the `arguments` representing argument nodes to some function.
    pub fn arguments(&self) -> Vec<SolAST> {
        let o = self.get_object();
        match o {
            None => vec![],
            Some(v) => {
                let arg = &v["arguments"].as_array();
                match arg {
                    Some(lst) => lst
                        .iter()
                        .map(|e| Self::new(e.clone(), self.contract.clone()))
                        .collect(),
                    None => vec![],
                }
            }
        }
    }

    /// Returns `statements` in some block.
    pub fn statements(&self) -> Vec<SolAST> {
        let o = self.get_object();
        match o {
            None => vec![],
            Some(v) => {
                let arg = &v["statements"].as_array();
                match arg {
                    Some(lst) => lst
                        .iter()
                        .map(|e| Self::new(e.clone(), self.contract.clone()))
                        .collect(),
                    None => vec![],
                }
            }
        }
    }

    /// Returns the `condition` field.
    pub fn condition(&self) -> SolAST {
        self.get_node("condition")
    }

    /// Returns the `trueBody` field.
    pub fn true_body(&self) -> SolAST {
        self.get_node("trueBody")
    }

    /// Returns the `falseBody` field.
    pub fn false_body(&self) -> SolAST {
        self.get_node("falseBody")
    }

    /// Returns the `typeDescriptions` field.
    pub fn get_type_descs(&self) -> Option<TypeDescriptions> {
        self.get_object()
            .map(|obj| TypeDescriptions::new(obj["typeDescriptions"].clone()))
    }

    /// Recursively traverses the AST.
    /// This is how
    /// Gambit determines what nodes can be mutated
    /// using which types of mutations and
    /// the exact location in the source where the mutation must be done.
    pub fn traverse<T, F>(
        self,
        mut visitor: F,
        mut skip: impl Fn(&SolAST) -> bool,
        mut accept: impl Fn(&SolAST) -> bool,
    ) -> Vec<T>
    where
        F: FnMut(&SolAST) -> Option<T>,
    {
        let mut result: Vec<T> = vec![];
        self.traverse_internal(&mut visitor, &mut skip, &mut accept, false, &mut result);
        result
    }

    fn traverse_internal<T>(
        mut self,
        visitor: &mut impl FnMut(&SolAST) -> Option<T>,
        skip: &mut impl FnMut(&SolAST) -> bool,
        accept: &mut impl FnMut(&SolAST) -> bool,
        accepted: bool,
        acc: &mut Vec<T>,
    ) {
        let mut new_accepted = accepted;
        if accept(&self) {
            new_accepted = true;
        }
        if skip(&self) {
            return;
        }
        if new_accepted {
            let res = visitor(&self);
            if let Some(r) = res {
                acc.push(r)
            } else {
                // log::info!("no mutation points found");
            }
        }
        if self.element.is_some() {
            let e = self.element.unwrap();
            if e.is_object() {
                let e_obj = e.as_object().unwrap();
                if e_obj.contains_key("contractKind") {
                    self.contract = e["name"].as_str().map(|nm| nm.to_string());
                }
                for v in e_obj.values() {
                    let child: SolAST = SolAST::new(v.clone(), self.contract.clone());
                    child.traverse_internal(visitor, skip, accept, new_accepted, acc);
                }
            } else if e.is_array() {
                let e_arr = e.as_array().unwrap();
                for a in e_arr {
                    let child: SolAST = SolAST::new(a.clone(), self.contract.clone());
                    child.traverse_internal(visitor, skip, accept, new_accepted, acc);
                }
            }
        }
    }

    /// Extracts the bounds from the AST that indicate where in the source
    /// a node's text starts and ends.
    /// This is represented by the `src` field in the AST about which more
    /// information can be found [here](https://docs.soliditylang.org/en/v0.8.17/using-the-compiler.html?highlight=--ast-compact--json#compiler-input-and-output-json-description).
    pub fn get_bounds(&self) -> (usize, usize) {
        let src = self.src().expect("Source information missing.");
        let parts: Vec<&str> = src.split(':').collect();
        let start = parts[0].parse::<usize>().unwrap();
        (start, start + parts[1].parse::<usize>().unwrap())
    }

    /// Returns the text corresponding to an AST node in the given `source`.
    pub fn get_text(&self, source: &[u8]) -> String {
        let (start, end) = self.get_bounds();
        let byte_vec = source[start..end].to_vec();
        String::from_utf8(byte_vec).expect("Slice is not u8.")
    }

    /// This method is used by a variety of mutations like `FunctionCallMutation`,
    /// `RequireMutation`, etc. (see more in `mutation.rs`) to directly
    /// mutate the source guided by information gathered from traversing the AST.
    pub fn replace_in_source(&self, source: &[u8], new: String) -> String {
        let (start, end) = self.get_bounds();
        self.replace_part(source, new, start, end)
    }

    /// This method is used to replace part of a statement.
    /// Example mutation types that use it are are `BinaryOperatorMutation`,
    /// `UnaryOperatorMutation`, and `ElimDelegateMutation`.
    pub fn replace_part(&self, source: &[u8], new: String, start: usize, end: usize) -> String {
        let before = &source[0..start];
        let changed = new.as_bytes();
        let after = &source[end..source.len()];
        let res = [before, changed, after].concat();
        String::from_utf8(res).expect("Slice is not u8.")
    }

    /// This method is used for "swap" mutations to swap lines of code,
    /// arguments to functions, or arguments to binary operators.
    /// See `MutationType` for more details on which mutantion types use this.
    pub fn replace_multiple(&self, source: &[u8], reps: Vec<(SolAST, String)>) -> String {
        let sorted = reps
            .iter()
            .map(|(node, n)| {
                let (s, e) = node.get_bounds();
                Replacement {
                    start: s,
                    end: e,
                    new: n.into(),
                }
            })
            .sorted_by_key(|x| x.start);
        let mut new_src = source.to_vec();
        let mut curr_offset = 0;
        for r in sorted {
            let actual_start = r.start + curr_offset;
            let actual_end = r.end + curr_offset;
            let replace_bytes = r.new.as_bytes();
            let new_start = &new_src[0..actual_start];
            let new_end = &new_src[actual_end..new_src.len()];
            new_src = [new_start, replace_bytes, new_end].concat();
            let new_offset = replace_bytes.len().wrapping_sub(r.end - r.start);
            curr_offset = curr_offset.wrapping_add(new_offset);
        }
        String::from_utf8(new_src.to_vec()).expect("Slice new_src is not u8.")
    }

    /// This method is used for mutations that comment out
    /// some piece of code using block comments.
    pub fn comment_out(&self, source: &[u8]) -> String {
        let (start, mut end) = self.get_bounds();
        let rest_of_str = String::from_utf8(source[end..source.len()].to_vec())
            .unwrap_or_else(|_| panic!("cannot convert bytes to string."));
        let mtch = Regex::new(r"^\*").unwrap().find(rest_of_str.as_str());
        if let Some(m) = mtch {
            end +=
                rest_of_str[0..m.range().last().unwrap_or_else(|| {
                    panic!("There was a match but last() still returned None.")
                }) + 1]
                    .as_bytes()
                    .len();
        }
        self.replace_part(
            source,
            "/*".to_string() + &String::from_utf8(source[start..end].to_vec()).unwrap() + "*/",
            start,
            end,
        )
    }
}
