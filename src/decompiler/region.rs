use crate::decompiler::ast::AstNode;
use std::slice::Iter;

/// Represents the type of control-flow region.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegionType {
    Linear,
    Condition,
    Tail,
}

/// Represents a region in the control-flow graph.
#[derive(Debug, Clone)]
pub struct Region {
    region_type: RegionType,
    statements: Vec<AstNode>,
}

impl Region {
    /// Creates a new region with the specified type and initializes with no statements.
    ///
    /// # Arguments
    ///
    /// * `region_type` - The type of the region.
    pub fn new(region_type: RegionType) -> Self {
        Self {
            region_type,
            statements: Vec::new(),
        }
    }

    /// Returns the type of the region.
    pub fn region_type(&self) -> &RegionType {
        &self.region_type
    }

    /// Adds a statement to the region.
    ///
    /// # Arguments
    ///
    /// * `node` - The AST node to add.
    pub fn push_instruction(&mut self, node: AstNode) {
        self.statements.push(node);
    }

    /// Returns an iterator over the statements in the region.
    pub fn iter_statements(&self) -> Iter<AstNode> {
        self.statements.iter()
    }
}

// === Implementations ===

/// Allows iterating over the statements in a region.
impl<'a> IntoIterator for &'a Region {
    type Item = &'a AstNode;
    type IntoIter = Iter<'a, AstNode>;

    fn into_iter(self) -> Self::IntoIter {
        self.statements.iter()
    }
}

/// Allows iterating over the statements in a region (mutable).
impl<'a> IntoIterator for &'a mut Region {
    type Item = &'a mut AstNode;
    type IntoIter = std::slice::IterMut<'a, AstNode>;

    fn into_iter(self) -> Self::IntoIter {
        self.statements.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decompiler::ast::bin_op::{BinOpType, BinaryOperationNode};
    use crate::decompiler::ast::expr::ExprNode;
    use crate::decompiler::ast::identifier::IdentifierNode;
    use crate::decompiler::ast::literal::LiteralNode;
    use crate::decompiler::ast::statement::StatementNode;

    fn create_identifier(id: &str) -> Box<ExprNode> {
        Box::new(ExprNode::Identifier(IdentifierNode::new(id.to_string())))
    }

    fn create_integer_literal(value: i32) -> Box<ExprNode> {
        Box::new(ExprNode::Literal(LiteralNode::Number(value)))
    }

    fn create_addition(lhs: Box<ExprNode>, rhs: Box<ExprNode>) -> Box<ExprNode> {
        Box::new(ExprNode::BinOp(
            BinaryOperationNode::new(lhs, rhs, BinOpType::Add).unwrap(),
        ))
    }

    fn create_subtraction(lhs: Box<ExprNode>, rhs: Box<ExprNode>) -> Box<ExprNode> {
        Box::new(ExprNode::BinOp(
            BinaryOperationNode::new(lhs, rhs, BinOpType::Sub).unwrap(),
        ))
    }

    fn create_statement(lhs: Box<ExprNode>, rhs: Box<ExprNode>) -> Box<StatementNode> {
        StatementNode::new(lhs, rhs).unwrap()
    }

    #[test]
    fn test_region_creation_and_instruction_addition() {
        let mut region = Region::new(RegionType::Linear);

        assert_eq!(region.region_type(), &RegionType::Linear);
        assert_eq!(region.iter_statements().count(), 0);

        let ast_node1 = create_statement(
            create_identifier("x"),
            create_addition(create_integer_literal(1), create_integer_literal(2)),
        );

        let ast_node2 = create_statement(
            create_identifier("y"),
            create_subtraction(create_integer_literal(3), create_integer_literal(4)),
        );

        region.push_instruction(AstNode::Statement(ast_node1.clone()));
        region.push_instruction(AstNode::Statement(ast_node2.clone()));

        let mut iter = region.iter_statements();
        assert_eq!(iter.next(), Some(&AstNode::Statement(ast_node1)));
        assert_eq!(iter.next(), Some(&AstNode::Statement(ast_node2)));
    }
}
