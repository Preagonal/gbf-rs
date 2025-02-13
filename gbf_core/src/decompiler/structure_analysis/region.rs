#![deny(missing_docs)]

use crate::cfg_dot::RenderableNode;
use crate::decompiler::ast::expr::ExprKind;
use crate::decompiler::ast::visitors::emit_context::{EmitContextBuilder, EmitVerbosity};
use crate::decompiler::ast::visitors::emitter::Gs2Emitter;
use crate::decompiler::ast::visitors::AstVisitor;
use crate::decompiler::ast::AstKind;
use crate::opcode::Opcode;
use crate::utils::{html_encode, GBF_GREEN, GBF_YELLOW};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Write};
use std::slice::Iter;

/// Represents the type of control-flow region.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Copy)]
pub enum RegionType {
    /// Simply moves on to the next region without control flow
    Linear,
    /// Control flow construct (e.g. for, while, if, switch, etc.)
    ControlFlow,
    /// A tail (e.g, return)
    Tail,
    /// A region that is inactive and removed from the graph from structure analysis
    Inactive,
}

/// Describes a region
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Copy)]
pub struct RegionId {
    /// The index of the region
    pub index: usize,
}

impl Display for RegionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RegionId({})", self.index)
    }
}

impl RegionId {
    /// Create a new `RegionId`.
    ///
    /// # Arguments
    /// - `index`: The index of the region in the graph.
    ///
    /// # Returns
    /// - A new `RegionId` instance.
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}

/// Represents a region in the control-flow graph.
#[derive(Debug, Clone)]
pub struct Region {
    nodes: Vec<AstKind>,
    unresolved_nodes: Vec<AstKind>,
    jump_expr: Option<ExprKind>,
    region_type: RegionType,
    branch_opcode: Option<Opcode>,
    region_id: RegionId,
}

impl Region {
    /// Creates a new region with the specified type and initializes with no statements.
    ///
    /// # Arguments
    /// * `id` - The id of the region.
    pub fn new(region_type: RegionType, region_id: RegionId) -> Self {
        Self {
            nodes: Vec::new(),
            unresolved_nodes: Vec::new(),
            jump_expr: None,
            region_type,
            branch_opcode: None,
            region_id,
        }
    }

    /// Returns the type of the region.
    pub fn region_type(&self) -> &RegionType {
        &self.region_type
    }

    /// Adds a statement to the region.
    ///
    /// # Arguments
    /// * `node` - The AST node to add.
    pub fn push_node(&mut self, node: AstKind) {
        self.nodes.push(node);
    }

    /// Adds an unresolved statement to the region.
    ///
    /// # Arguments
    /// * `node` - The AST node to add.
    pub fn push_unresolved_node(&mut self, node: AstKind) {
        self.unresolved_nodes.push(node);
    }

    /// Adds multiple statements to the region.
    ///
    /// # Arguments
    /// * `nodes` - The AST nodes to add.
    pub fn push_nodes(&mut self, nodes: Vec<AstKind>) {
        self.nodes.extend(nodes);
    }

    /// Adds multiple unresolved statements to the region.
    ///
    /// # Arguments
    /// * `nodes` - The AST nodes to add.
    pub fn push_unresolved_nodes(&mut self, nodes: Vec<AstKind>) {
        self.unresolved_nodes.extend(nodes);
    }

    /// Clears the nodes in the region
    pub fn clear_nodes(&mut self) {
        self.nodes.clear();
    }

    /// Clears the unresolved nodes in the region
    pub fn clear_unresolved_nodes(&mut self) {
        self.unresolved_nodes.clear();
    }

    /// Gets the nodes in the region.
    ///
    /// # Return
    /// The nodes in the region.
    pub fn get_nodes(&self) -> &Vec<AstKind> {
        &self.nodes
    }

    /// Gets the unresolved nodes in the region.
    ///
    /// # Return
    /// The unresolved nodes in the region.
    pub fn get_unresolved_nodes(&self) -> &Vec<AstKind> {
        &self.unresolved_nodes
    }

    /// Gets the region type.
    ///
    /// # Return
    /// The region type.
    pub fn get_region_type(&self) -> RegionType {
        self.region_type
    }

    /// Sets the type of the region.
    ///
    /// # Arguments
    /// * `region_type` - The new type of the region.
    pub fn set_region_type(&mut self, region_type: RegionType) {
        self.region_type = region_type;
    }

    /// Remove the jump expression from the region.
    pub fn remove_jump_expr(&mut self) {
        self.jump_expr = None;
    }

    /// Gets the jump expression.
    ///
    /// # Return
    /// The jump expression.
    pub fn get_jump_expr(&self) -> Option<&ExprKind> {
        self.jump_expr.as_ref()
    }

    /// Sets the jump expression.
    ///
    /// # Arguments
    /// * `jump_expr` - The new optional jump expression.
    pub fn set_jump_expr(&mut self, jump_expr: Option<ExprKind>) {
        self.jump_expr = jump_expr;
    }

    /// Gets the branch opcode, if any.
    ///
    /// # Return
    /// The opcode.
    pub fn get_branch_opcode(&self) -> Option<Opcode> {
        self.branch_opcode
    }

    /// Sets the opcode.
    ///
    /// # Arguments
    /// * `opcode` - The new opcode.
    pub fn set_branch_opcode(&mut self, opcode: Opcode) {
        self.branch_opcode = Some(opcode);
    }

    /// Returns an iterator over the statements in the region.
    ///
    /// # Return
    /// An iterator over the statements in the region.
    pub fn iter_nodes(&self) -> Iter<AstKind> {
        self.nodes.iter()
    }
}

// === Other Implementations ===

/// Allows iterating over the statements in a region.
impl<'a> IntoIterator for &'a Region {
    type Item = &'a AstKind;
    type IntoIter = Iter<'a, AstKind>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter()
    }
}

/// Allows iterating over the statements in a region (mutable).
impl<'a> IntoIterator for &'a mut Region {
    type Item = &'a mut AstKind;
    type IntoIter = std::slice::IterMut<'a, AstKind>;

    fn into_iter(self) -> Self::IntoIter {
        self.nodes.iter_mut()
    }
}

impl RenderableNode for Region {
    /// Render the region's node representation for Graphviz with customizable padding.
    ///
    /// # Arguments
    /// * `padding` - The number of spaces to use for indentation.
    ///
    /// # Return
    /// The rendered node
    fn render_node(&self, padding: usize) -> String {
        let mut label = String::new();
        let indent = " ".repeat(padding);

        // Start the HTML-like table for Graphviz.
        writeln!(
            &mut label,
            r#"{indent}<TABLE BORDER="0" CELLBORDER="0" CELLSPACING="0" CELLPADDING="0">"#,
            indent = indent
        )
        .unwrap();

        // Write RegionId
        writeln!(
            &mut label,
            r#"{indent}<TR><TD ALIGN="LEFT"><FONT COLOR="{GBF_GREEN}">{}</FONT></TD></TR><TR><TD> </TD></TR>"#,
            html_encode(format!("{}", self.region_id)),
            GBF_GREEN = GBF_GREEN,
            indent = indent
        ).unwrap();

        // Write the region type as the label.
        let region_type_str = match self.region_type {
            RegionType::Linear => "Linear",
            RegionType::ControlFlow => "ControlFlow",
            RegionType::Tail => "Tail",
            RegionType::Inactive => "Inactive",
        };
        writeln!(
            &mut label,
            r#"{indent}<TR><TD ALIGN="LEFT"><FONT COLOR="{GBF_GREEN}">RegionType: {}</FONT></TD></TR><TR><TD> </TD></TR>"#,
            region_type_str,
            GBF_GREEN = GBF_GREEN,
            indent = indent
        )
        .unwrap();

        // Render each statement as a table row with indentation.
        for node in &self.nodes {
            // Build a new EmitContext with debug
            let context = EmitContextBuilder::default()
                .verbosity(EmitVerbosity::Debug)
                .include_ssa_versions(true)
                .build();
            let mut emitter = Gs2Emitter::new(context);
            let result = emitter.visit_node(node).node;

            // split the result by newlines
            let result = result.split('\n').collect::<Vec<&str>>();

            for line in result {
                writeln!(
                    &mut label,
                    r#"{indent}<TR><TD ALIGN="LEFT"><FONT COLOR="{GBF_YELLOW}">{}</FONT></TD></TR>"#,
                    html_encode(line),
                    GBF_YELLOW = GBF_YELLOW,
                    indent = indent
                )
                .unwrap();
            }
        }

        // If the region has a condition, render it.
        if let Some(jump_expr) = &self.jump_expr {
            // Build a new EmitContext with debug
            let context = EmitContextBuilder::default()
                .verbosity(EmitVerbosity::Pretty)
                .include_ssa_versions(true)
                .build();
            let mut emitter = Gs2Emitter::new(context);
            let result = emitter.visit_expr(jump_expr);

            writeln!(
                &mut label,
                r##"{indent}    <TR><TD> </TD></TR><TR>
{indent}        <TD ALIGN="LEFT"><FONT COLOR="{GBF_GREEN}">JumpExpr: {}</FONT></TD>
{indent}    </TR>"##,
                html_encode(result.node),
                GBF_GREEN = GBF_GREEN,
                indent = indent
            )
            .unwrap();
        }

        // Close the HTML-like table.
        writeln!(&mut label, "{indent}</TABLE>", indent = indent).unwrap();

        label
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decompiler::ast::{bin_op::BinOpType, new_assignment, new_bin_op, new_id, new_num};

    #[test]
    fn test_region_creation_and_instruction_addition() {
        let mut region = Region::new(RegionType::Linear, RegionId::new(0));

        assert_eq!(region.region_type(), &RegionType::Linear);
        assert_eq!(region.iter_nodes().count(), 0);

        let ast_node1 = new_assignment(
            new_id("x"),
            new_bin_op(new_num(1), new_num(2), BinOpType::Add).unwrap(),
        );

        let ast_node2 = new_assignment(
            new_id("y"),
            new_bin_op(new_num(3), new_num(4), BinOpType::Sub).unwrap(),
        );

        region.push_node(ast_node1.clone().into());
        region.push_node(ast_node2.clone().into());

        let mut iter = region.iter_nodes();
        assert_eq!(iter.next(), Some(&ast_node1.clone().into()));
        assert_eq!(iter.next(), Some(&ast_node2.clone().into()));
    }

    #[test]
    fn test_region_into_iter() {
        let region = Region::new(RegionType::Linear, RegionId::new(1));
        let mut iter = region.into_iter();
        assert_eq!(iter.next(), None);
    }
}
