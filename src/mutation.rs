use crate::SolAST;
use clap::ValueEnum;
use rand::{seq::SliceRandom, RngCore};
use rand_pcg::*;
use serde::{Deserialize, Serialize};

/// Every kind of mutation implements this trait.
///
/// `is_mutation_point` determines whether a node in the AST
/// is a valid node for performing a certain `MutationType`.
///
/// `mutate_randomly` mutates such nodes by randomly selecting
/// one of many possible ways to perform `MutationType`.
///
/// For example, consider the `BinaryOpMutation` `MutationType`.
/// The method `is_mutation_point` for this mutation checks where the
/// node under question has the `node_type` `BinaryOperation`.
///
/// `mutate_randomly` for this mutation will randomly pick one
/// of many binary operators supported in Solidity (e.g., +, -, *, /, **, ...])
/// and apply it at the source location of the original binary operator.
///
pub trait Mutation {
    fn is_mutation_point(&self, node: &SolAST) -> bool;
    fn mutate_randomly(&self, node: &SolAST, source: &[u8], rand: &mut Pcg64) -> String;
}

/// Kinds of mutations.
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug, ValueEnum, Deserialize, Serialize)]
pub enum MutationType {
    BinaryOpMutation,
    RequireMutation,
    AssignmentMutation,
    DeleteExpressionMutation,
    FunctionCallMutation,
    IfStatementMutation,
    SwapArgumentsFunctionMutation,
    SwapArgumentsOperatorMutation,
    SwapLinesMutation,
    UnaryOperatorMutation,
    ElimDelegateMutation,
}

impl ToString for MutationType {
    fn to_string(&self) -> String {
        let str = match self {
            MutationType::BinaryOpMutation => "BinaryOpMutation",
            MutationType::RequireMutation => "RequireMutation",
            MutationType::AssignmentMutation => "AssignmentMutation",
            MutationType::DeleteExpressionMutation => "DeleteExpressionMutation",
            MutationType::FunctionCallMutation => "FunctionCallMutation",
            MutationType::IfStatementMutation => "IfStatementMutation",
            MutationType::SwapArgumentsFunctionMutation => "SwapArgumentsFunctionMutation",
            MutationType::SwapArgumentsOperatorMutation => "SwapArgumentsOperatorMutation",
            MutationType::SwapLinesMutation => "SwapLinesMutation",
            MutationType::UnaryOperatorMutation => "UnaryOperatorMutation",
            MutationType::ElimDelegateMutation => "ElimDelegateMutation",
        };
        str.into()
    }
}

impl Mutation for MutationType {
    fn is_mutation_point(&self, node: &SolAST) -> bool {
        match self {
            MutationType::BinaryOpMutation => {
                if let Some(n) = node.node_type() {
                    return n == "BinaryOperation";
                }
            }
            MutationType::RequireMutation => {
                return node.node_type().map_or_else(
                    || false,
                    |n| {
                        n == "FunctionCall"
                            && (node
                                .expression()
                                .name()
                                .map_or_else(|| false, |nm| nm == "require"))
                            && !node.arguments().is_empty()
                    },
                );
            }
            MutationType::AssignmentMutation => {
                if let Some(n) = node.node_type() {
                    return n == "Assignment";
                }
            }
            MutationType::DeleteExpressionMutation => {
                if let Some(n) = node.node_type() {
                    return n == "ExpressionStatement";
                }
            }
            MutationType::FunctionCallMutation => {
                if let Some(n) = node.node_type() {
                    return n == "FunctionCall" && !node.arguments().is_empty();
                }
            }
            MutationType::IfStatementMutation => {
                if let Some(n) = node.node_type() {
                    return n == "IfStatement";
                }
            }
            MutationType::SwapArgumentsFunctionMutation => {
                if let Some(n) = node.node_type() {
                    return n == "FunctionCall" && node.arguments().len() > 1;
                }
            }
            MutationType::SwapArgumentsOperatorMutation => {
                let non_comm_ops = vec!["-", "/", "%", "**", ">", "<", ">=", "<=", "<<", ">>"];
                if let Some(n) = node.node_type() {
                    return n == "BinaryOperation"
                        && non_comm_ops.contains(
                            &node
                                .operator()
                                .unwrap_or_else(|| panic!("Expression does not have operator"))
                                .as_str(),
                        );
                }
            }
            MutationType::SwapLinesMutation => {
                if let Some(n) = node.node_type() {
                    return n == "Block" && node.statements().len() > 1;
                }
            }
            MutationType::UnaryOperatorMutation => {
                if let Some(n) = node.node_type() {
                    return n == "UnaryOperation";
                }
            }
            MutationType::ElimDelegateMutation => {
                return node.node_type().map_or_else(
                    || false,
                    |n| {
                        n == "FunctionCall"
                            && (node
                                .expression()
                                .node_type()
                                .map_or_else(|| false, |nt| nt == "MemberAccess"))
                            && (node
                                .expression()
                                .get_string("memberName")
                                .map_or_else(|| false, |mn| mn == "delegatecall"))
                    },
                );
            }
        }
        false
    }

    fn mutate_randomly(&self, node: &SolAST, source: &[u8], rand: &mut Pcg64) -> String {
        match self {
            MutationType::BinaryOpMutation => {
                assert!(&self.is_mutation_point(node));
                let ops = vec!["+", "-", "*", "/", "%", "**"];
                let (_, endl) = node.left_expression().get_bounds();
                let (startr, _) = node.right_expression().get_bounds();
                node.replace_part(
                    source,
                    " ".to_string() + ops.choose(rand).unwrap() + " ",
                    endl,
                    startr,
                )
            }
            MutationType::RequireMutation => {
                assert!(&self.is_mutation_point(node));
                let arg = &node.arguments()[0];
                arg.replace_in_source(source, "!(".to_string() + &arg.get_text(source) + ")")
            }
            MutationType::DeleteExpressionMutation => {
                assert!(&self.is_mutation_point(node));
                node.comment_out(source)
            }
            MutationType::FunctionCallMutation => {
                assert!(&self.is_mutation_point(node));
                if let Some(arg) = node.arguments().choose(rand) {
                    node.replace_in_source(source, arg.get_text(source))
                } else {
                    node.get_text(source)
                }
            }
            MutationType::IfStatementMutation => {
                assert!(&self.is_mutation_point(node));
                let cond = node.condition();
                let bs = vec![true, false];
                if *bs.choose(rand).unwrap() {
                    cond.replace_in_source(source, (*bs.choose(rand).unwrap()).to_string())
                } else {
                    cond.replace_in_source(source, "!(".to_owned() + &cond.get_text(source) + ")")
                }
            }
            MutationType::SwapArgumentsFunctionMutation => {
                assert!(&self.is_mutation_point(node));
                let mut children = node.arguments();
                children.shuffle(rand);
                if children.len() == 2 {
                    node.replace_multiple(
                        source,
                        vec![
                            (children[0].clone(), children[1].get_text(source)),
                            (children[1].clone(), children[0].get_text(source)),
                        ],
                    )
                } else {
                    node.get_text(source)
                }
            }
            MutationType::SwapArgumentsOperatorMutation => {
                assert!(&self.is_mutation_point(node));
                let left = node.left_expression();
                let right = node.right_expression();
                node.replace_multiple(
                    source,
                    vec![
                        (left.clone(), right.get_text(source)),
                        (right, left.get_text(source)),
                    ],
                )
            }
            MutationType::SwapLinesMutation => {
                assert!(&self.is_mutation_point(node));
                let mut stmts = node.statements();
                stmts.shuffle(rand);
                if stmts.len() == 2 {
                    node.replace_multiple(
                        source,
                        vec![
                            (stmts[0].clone(), stmts[1].get_text(source)),
                            (stmts[1].clone(), stmts[0].get_text(source)),
                        ],
                    )
                } else {
                    node.get_text(source)
                }
            }
            MutationType::UnaryOperatorMutation => {
                assert!(&self.is_mutation_point(node));
                let prefix_ops = vec!["++", "--", "~"];
                let suffix_ops = vec!["++", "--"];
                let is_prefix =
                    |source: &[u8], op: &String| -> bool { return source[0] == op.as_bytes()[0] };
                let (start, end) = node.get_bounds();
                let op = node
                    .operator()
                    .expect("Unary operation must have an operator!");
                return if is_prefix(source, &op) {
                    node.replace_part(
                        source,
                        prefix_ops.choose(rand).unwrap().to_string(),
                        start,
                        start + op.len(),
                    )
                } else {
                    node.replace_part(
                        source,
                        suffix_ops.choose(rand).unwrap().to_string(),
                        end - op.len(),
                        end,
                    )
                };
            }
            MutationType::AssignmentMutation => {
                assert!(&self.is_mutation_point(node));
                let new: Vec<String> =
                    vec!["true", "false", "0", "1", &rand.next_u64().to_string()]
                        .iter()
                        .map(|e| e.to_string())
                        .collect();
                let rhs = node.right_hand_side();
                match rhs.element {
                    Some(_) => rhs.replace_in_source(source, new.choose(rand).unwrap().to_string()),
                    None => panic!("No rhs for this assignment!"),
                }
            }
            MutationType::ElimDelegateMutation => {
                assert!(&self.is_mutation_point(node));
                let (_, endl) = node.expression().expression().get_bounds();
                let (_, endr) = node.expression().get_bounds();
                node.replace_part(source, "call".to_string(), endl + 1, endr)
            }
        }
    }
}
