#![deny(missing_docs)]

use crate::decompiler::ast::visitors::AstVisitor;
use array_access::ArrayAccessNode;
use assignable::AssignableKind;
use assignment::AssignmentNode;
use ast_vec::AstVec;
use bin_op::BinaryOperationNode;
use block::BlockNode;
use control_flow::{ControlFlowNode, ControlFlowType};
use expr::ExprKind;
use func_call::FunctionCallNode;
use function::FunctionNode;
use identifier::IdentifierNode;
use literal::LiteralNode;
use member_access::MemberAccessNode;
use meta::MetaNode;
use ret::ReturnNode;
use serde::{Deserialize, Serialize};
use ssa::SsaVersion;
use statement::StatementKind;
use thiserror::Error;
use unary_op::UnaryOperationNode;
use visitors::{emit_context::EmitContext, emitter::Gs2Emitter};

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
/// Represents an AST vector.
pub mod ast_vec;
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
/// Represents a return node in the AST.
pub mod ret;
/// Represents SSA versioning for the AST.
pub mod ssa;
/// Represents a statement node in the AST.
pub mod statement;
/// Represents unary operations in the AST.
pub mod unary_op;
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
    /// Clones the AST node as a boxed trait object.
    fn clone_box(&self) -> Box<Self>
    where
        Self: Sized,
    {
        Box::new(self.clone())
    }

    /// Accepts a visitor for the AST node.
    fn accept(&self, visitor: &mut dyn AstVisitor);
}

/// Represents an AST node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AstKind {
    /// Represents a statement node in the AST, such as `variable = value;`.
    Statement(StatementKind),
    /// Represents a function node in the AST.
    Function(FunctionNode),
    /// Represents an expression node in the AST.
    Expression(ExprKind),
    /// Represents a metadata node in the AST.
    Meta(MetaNode), // Covers comments or annotations
    /// Represenst a block of code in the AST.
    Block(BlockNode),
    /// Represents a control flow node in the AST.
    ControlFlow(ControlFlowNode),
}

impl AstVisitable for AstKind {
    fn accept(&self, visitor: &mut dyn AstVisitor) {
        match self {
            AstKind::Expression(expr) => expr.accept(visitor),
            AstKind::Meta(meta) => meta.accept(visitor),
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
    node.accept(&mut emit);
    emit.output().to_string()
}

// = Assignable expressions =

/// Creates a metadata node with a comment
pub fn new_comment<N>(node: N, comment: &str) -> MetaNode
where
    N: Into<AstKind>,
{
    MetaNode::new(
        node.into().into(),
        Some(comment.to_string()),
        None,
        Default::default(),
    )
}

/// Creates a new AstNode for a statement.
pub fn new_assignment<L, R>(lhs: L, rhs: R) -> AssignmentNode
where
    L: Into<Box<AssignableKind>>,
    R: Into<Box<ExprKind>>,
{
    AssignmentNode {
        lhs: lhs.into(),
        rhs: rhs.into(),
    }
}

/// Creates a new return node.
pub fn new_return<N>(node: N) -> ReturnNode
where
    N: Into<Box<ExprKind>>,
{
    ReturnNode::new(node.into())
}

/// Creates a new member access node.
pub fn new_member_access<L, R>(lhs: L, rhs: R) -> Result<MemberAccessNode, AstNodeError>
where
    L: Into<Box<AssignableKind>>,
    R: Into<Box<AssignableKind>>,
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
pub fn new_fn_call<N, A>(name: N, args: A) -> FunctionCallNode
where
    N: Into<AssignableKind>,
    A: Into<AstVec<ExprKind>>,
{
    FunctionCallNode::new(name.into(), args.into())
}

/// Creates a new array node.
pub fn new_array<A>(elements: A) -> array::ArrayNode
where
    A: Into<AstVec<ExprKind>>,
{
    array::ArrayNode::new(elements.into())
}

/// Creates a new array access node.
pub fn new_array_access<A, I>(array: A, index: I) -> ArrayAccessNode
where
    A: Into<Box<AssignableKind>>,
    I: Into<Box<ExprKind>>,
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
    L: Into<Box<ExprKind>>,
    R: Into<Box<ExprKind>>,
{
    BinaryOperationNode::new(lhs.into(), rhs.into(), op_type)
}

/// Creates a new unary operation node.
pub fn new_unary_op<A>(
    operand: A,
    op_type: unary_op::UnaryOpType,
) -> Result<UnaryOperationNode, AstNodeError>
where
    A: Into<Box<ExprKind>>,
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

// == Functions ==
/// Creates a new function node.
pub fn new_fn<P, V>(name: Option<String>, params: P, body: V) -> FunctionNode
where
    P: Into<AstVec<ExprKind>>,
    V: Into<AstVec<AstKind>>,
{
    FunctionNode::new(name, params.into(), body)
}

// == Conditionals ==
/// Creates a new if statement
pub fn new_if<C, T>(condition: C, then_block: T) -> ControlFlowNode
where
    C: Into<ExprKind>,
    T: Into<AstVec<AstKind>>,
{
    ControlFlowNode::new(ControlFlowType::If, Some(condition), then_block.into())
}

/// Creates a new else statement
pub fn new_else<T>(else_block: T) -> ControlFlowNode
where
    T: Into<AstVec<AstKind>>,
{
    ControlFlowNode::new(ControlFlowType::Else, None::<ExprKind>, else_block.into())
}

/// Creates a new else if statement
pub fn new_else_if<C, T>(condition: C, then_block: T) -> ControlFlowNode
where
    C: Into<ExprKind>,
    T: Into<AstVec<AstKind>>,
{
    ControlFlowNode::new(ControlFlowType::ElseIf, Some(condition), then_block.into())
}
