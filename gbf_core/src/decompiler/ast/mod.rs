#![deny(missing_docs)]

use crate::{decompiler::ast::visitors::AstVisitor, opcode::Opcode};
use array_access::ArrayAccessNode;
use assignable::AssignableKind;
use assignment::AssignmentNode;
use bin_op::BinaryOperationNode;
use block::BlockNode;
use control_flow::{ControlFlowNode, ControlFlowType};
use expr::ExprKind;
use func_call::FunctionCallNode;
use function::FunctionNode;
use identifier::IdentifierNode;
use literal::LiteralNode;
use member_access::MemberAccessNode;
use phi::PhiNode;
use ptr::P;
use ret::ReturnNode;
use serde::{Deserialize, Serialize};
use ssa::SsaVersion;
use statement::StatementKind;
use thiserror::Error;
use unary_op::UnaryOperationNode;
use vbranch::VirtualBranchNode;
use visitors::{emit_context::EmitContext, emitter::Gs2Emitter};

use super::structure_analysis::region::RegionId;

/// Represents an array
pub mod array;
/// Represents an array access node.
pub mod array_access;
/// Contains the specifications for any AstNodes that are assignable.
pub mod assignable;
/// Contains the specifications for any AstNodes that are assignments
pub mod assignment;
/// Holds the macro that generates variants for the AST nodes.
pub mod ast_enum_type;
/// Represents binary operations in the AST.
pub mod bin_op;
/// Represents a "block" of code in the AST.
pub mod block;
/// Represents a control flow node in the AST.
pub mod control_flow;
/// Contains the specifications for any AstNodes that are expressions
pub mod expr;
/// Contains the specifications for any AstNodes that are function calls.
pub mod func_call;
/// Contains the specifications for any AstNodes that are functions.
pub mod function;
/// Contains the specifications for any AstNodes that are identifiers.
pub mod identifier;
/// Contains the specifications for any AstNodes that are literals.
pub mod literal;
/// Contains the specifications for any AstNodes that are member accesses.
pub mod member_access;
/// Contains the specifications for any AstNodes that are metadata.
pub mod meta;
/// Represents the new
pub mod new;
/// A node identifier
pub mod node_id;
/// Represents a phi node in the AST.
pub mod phi;
/// Represents a pointer
pub mod ptr;
/// Represents a return node in the AST.
pub mod ret;
/// Represents SSA versioning for the AST.
pub mod ssa;
/// Represents a statement node in the AST.
pub mod statement;
/// Represents unary operations in the AST.
pub mod unary_op;
/// Represents a virtual branch
pub mod vbranch;
/// Represents the visitor pattern for the AST.
pub mod visitors;

/// Represents an error that occurred while converting an AST node.
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum AstNodeError {
    /// Invalid conversion from AstNode to another type.
    #[error("Expected {0}, found {1}")]
    InvalidConversion(String, String),

    /// Invalid operand for an AST node.
    #[error("Invalid operand type")]
    InvalidOperand,

    /// Cannot invert the AST node.
    #[error("Cannot invert {0}")]
    CannotInvert(String),
}

/// Trait for all AST nodes.
pub trait AstVisitable: Clone {
    /// Accepts a visitor for the AST node.
    fn accept<V: AstVisitor>(&self, visitor: &mut V) -> V::Output;
}

/// Represents an AST node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AstKind {
    /// Represents a statement node in the AST, such as `variable = value;`.
    Statement(StatementKind),
    /// Represents a function node in the AST.
    Function(P<FunctionNode>),
    /// Represents an expression node in the AST.
    Expression(ExprKind),
    /// Represenst a block of code in the AST.
    Block(P<BlockNode>),
    /// Represents a control flow node in the AST.
    ControlFlow(P<ControlFlowNode>),
}

impl AstVisitable for AstKind {
    fn accept<V: AstVisitor>(&self, visitor: &mut V) -> V::Output {
        match self {
            AstKind::Expression(expr) => expr.accept(visitor),
            AstKind::Statement(stmt) => stmt.accept(visitor),
            AstKind::Function(func) => func.accept(visitor),
            AstKind::Block(block) => block.accept(visitor),
            AstKind::ControlFlow(control_flow) => control_flow.accept(visitor),
        }
    }
}

/// Emits a node into a string.
pub fn emit<N>(node: N) -> String
where
    N: Into<AstKind>,
{
    let node: AstKind = node.into();
    let mut emit = Gs2Emitter::new(EmitContext::default());
    let ouput = node.accept(&mut emit);
    ouput.node
}

// = Assignable expressions =

/// Creates a new AstNode for a statement.
pub fn new_assignment<L, R>(lhs: L, rhs: R) -> AssignmentNode
where
    L: Into<AssignableKind>,
    R: Into<ExprKind>,
{
    AssignmentNode {
        lhs: lhs.into(),
        rhs: rhs.into(),
    }
}

/// Creates a new return node.
pub fn new_return<N>(node: N) -> ReturnNode
where
    N: Into<ExprKind>,
{
    ReturnNode::new(node.into())
}

/// Creates a new virtual branch node.
pub fn new_virtual_branch(branch: RegionId) -> VirtualBranchNode {
    VirtualBranchNode::new(branch)
}

/// Creates a new member access node.
pub fn new_member_access<L, R>(lhs: L, rhs: R) -> Result<MemberAccessNode, AstNodeError>
where
    L: Into<AssignableKind>,
    R: Into<AssignableKind>,
{
    MemberAccessNode::new(lhs.into(), rhs.into())
}

// = Expressions =

/// Creates a new AssignableExpr for an identifier
pub fn new_id(name: &str) -> IdentifierNode {
    IdentifierNode::new(name)
}

/// Creates a new AssignableExpr for an identifier with an SSA version.
pub fn new_id_with_version(name: &str, version: SsaVersion) -> IdentifierNode {
    IdentifierNode::with_ssa(name, version)
}

/// Creates a new function call node.
pub fn new_fn_call<N>(name: N, args: Vec<ExprKind>) -> FunctionCallNode
where
    N: Into<AssignableKind>,
{
    FunctionCallNode::new(name.into(), args)
}

/// Creates a new array node.
pub fn new_array<E>(elements: Vec<E>) -> array::ArrayNode
where
    E: Into<ExprKind>,
{
    array::ArrayNode::new(elements.into_iter().map(Into::into).collect())
}

/// Creates a new array access node.
pub fn new_array_access<A, I>(array: A, index: I) -> ArrayAccessNode
where
    A: Into<AssignableKind>,
    I: Into<ExprKind>,
{
    ArrayAccessNode::new(array.into(), index.into())
}

/// Creates binary operation node.
pub fn new_bin_op<L, R>(
    lhs: L,
    rhs: R,
    op_type: bin_op::BinOpType,
) -> Result<BinaryOperationNode, AstNodeError>
where
    L: Into<ExprKind>,
    R: Into<ExprKind>,
{
    BinaryOperationNode::new(lhs.into(), rhs.into(), op_type)
}

/// Creates a new unary operation node.
pub fn new_unary_op<A>(
    operand: A,
    op_type: unary_op::UnaryOpType,
) -> Result<UnaryOperationNode, AstNodeError>
where
    A: Into<ExprKind>,
{
    UnaryOperationNode::new(operand.into(), op_type)
}

// == Literals ==

/// Creates a new ExprNode for a literal string.
pub fn new_str(value: &str) -> LiteralNode {
    LiteralNode::String(value.to_string())
}

/// Creates a new ExprNode for a literal number.
pub fn new_num(value: i32) -> LiteralNode {
    LiteralNode::Number(value)
}

/// Creates a new ExprNode for a literal float.
pub fn new_float(value: &str) -> LiteralNode {
    LiteralNode::Float(value.to_string())
}

/// Creates a new ExprNode for a literal boolean.
pub fn new_bool(value: bool) -> LiteralNode {
    LiteralNode::Boolean(value)
}

/// Creates a new ExprNode for a literal null.
pub fn new_null() -> LiteralNode {
    LiteralNode::Null
}

// == Functions ==
/// Creates a new function node.
pub fn new_fn<V, E>(name: Option<String>, params: Vec<E>, body: Vec<V>) -> FunctionNode
where
    V: Into<AstKind>,
    E: Into<ExprKind>,
{
    FunctionNode::new(
        name,
        params.into_iter().map(Into::into).collect(),
        body.into_iter().map(Into::into).collect::<Vec<AstKind>>(),
    )
}

// == Conditionals ==
/// Creates a new if statement
pub fn new_if<C, E>(condition: C, then_block: Vec<E>) -> ControlFlowNode
where
    C: Into<ExprKind>,
    E: Into<AstKind>,
{
    ControlFlowNode::new(
        ControlFlowType::If,
        Some(condition),
        then_block
            .into_iter()
            .map(Into::into)
            .collect::<Vec<AstKind>>(),
    )
}

/// Creates a new else statement
pub fn new_else<T>(else_block: Vec<T>) -> ControlFlowNode
where
    T: Into<AstKind>,
{
    ControlFlowNode::new(
        ControlFlowType::Else,
        None::<ExprKind>,
        else_block
            .into_iter()
            .map(Into::into)
            .collect::<Vec<AstKind>>(),
    )
}

/// Creates a new with statement
pub fn new_with<C, T>(condition: C, then_block: Vec<T>) -> ControlFlowNode
where
    C: Into<ExprKind>,
    T: Into<AstKind>,
{
    ControlFlowNode::new(
        ControlFlowType::With,
        Some(condition),
        then_block
            .into_iter()
            .map(Into::into)
            .collect::<Vec<AstKind>>(),
    )
}

// == Phi Nodes ==
/// Creates a new phi node.
pub fn new_phi(index: usize) -> phi::PhiNode {
    PhiNode::new(index)
}

/// Creates a new acyclic condition
pub fn new_acylic_condition<C, T>(
    condition: C,
    then_block: Vec<T>,
    opcode: Option<Opcode>,
) -> Result<ControlFlowNode, AstNodeError>
where
    C: Into<ExprKind>,
    T: Into<AstKind>,
{
    match opcode {
        Some(Opcode::Jne) => Ok(new_if(
            condition,
            then_block
                .into_iter()
                .map(Into::into)
                .collect::<Vec<AstKind>>(),
        )),
        // TODO: Move condition flipping logic here for Jeq
        Some(Opcode::Jeq) => Ok(new_if(condition, then_block)),
        // TODO: We may need to flip the condition on ShortCircuitAnd or ShortCircuitOr
        Some(Opcode::ShortCircuitAnd) => Ok(new_if(condition, then_block)),
        Some(Opcode::ShortCircuitOr) => Ok(new_if(condition, then_block)),
        Some(Opcode::With) => Ok(new_with(condition, then_block)),
        None => Ok(new_if(condition, then_block)),
        _ => Err(AstNodeError::InvalidOperand),
    }
}

/// Creates a new new node.
pub fn new_new<N>(new_type: &str, arg: N) -> Result<new::NewNode, AstNodeError>
where
    N: Into<ExprKind>,
{
    new::NewNode::new(new_type, arg.into())
}
