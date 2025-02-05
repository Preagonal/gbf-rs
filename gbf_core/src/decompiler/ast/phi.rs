#![deny(missing_docs)]

use gbf_macros::AstNodeTransform;
use serde::{Deserialize, Serialize};

use crate::decompiler::structure_analysis::{region::RegionId, ControlFlowEdgeType};

use super::{
    assignable::AssignableKind, expr::ExprKind, ptr::P, ssa::SsaVersion, visitors::AstVisitor,
    AstKind, AstVisitable,
};

/// Represents a Phi node in SSA form.
///
/// Phi nodes are used to merge values coming from different control-flow paths.
/// Initially, the phi node has no arguments (i.e. no predecessor regions), but you
/// can add them later using the [`add_region`] method.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, AstNodeTransform)]
#[convert_to(AstKind::Expression, ExprKind::Assignable, AssignableKind::Phi)]
pub struct PhiNode {
    region_ids: Vec<(RegionId, ControlFlowEdgeType)>,
    /// Represents the SSA version of a variable.
    pub ssa_version: Option<SsaVersion>,
    /// Index of the phi node, cooresponding to the index of the execution frame.
    pub index: usize,
}

impl PhiNode {
    /// Creates a new `PhiNode` with no predecessor regions.
    ///
    /// # Returns
    ///
    /// A new phi node with an empty list of region IDs.
    pub fn new(index: usize) -> Self {
        Self {
            region_ids: Vec::new(),
            ssa_version: None,
            index,
        }
    }

    /// Adds a predecessor `RegionId` to this phi node.
    ///
    /// This method allows the phi node to record a region (i.e. a basic block ID)
    /// from which a value is coming.
    ///
    /// # Arguments
    /// * `region` - The identifier of the predecessor region.
    pub fn add_region(&mut self, region: RegionId, edge_type: ControlFlowEdgeType) {
        self.region_ids.push((region, edge_type));
    }

    /// Adds predecessor `RegionId`s to this phi node.
    ///
    /// This method allows the phi node to record multiple regions (i.e. basic block IDs)
    /// from which values are coming.
    ///
    /// # Arguments
    /// * `regions` - The identifiers of the predecessor regions.
    pub fn add_regions(&mut self, regions: Vec<(RegionId, ControlFlowEdgeType)>) {
        self.region_ids.extend(regions);
    }

    /// Returns a reference to the list of region IDs associated with this phi node.
    ///
    /// # Returns
    ///
    /// A slice containing all the predecessor region IDs added so far.
    pub fn regions(&self) -> &[(RegionId, ControlFlowEdgeType)] {
        &self.region_ids
    }
}

impl AstVisitable for P<PhiNode> {
    fn accept<V: AstVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_phi(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::decompiler::structure_analysis::{region::RegionId, ControlFlowEdgeType};

    use super::PhiNode;

    #[test]
    fn test_phi_node_add_region() {
        let mut phi = PhiNode::new(0);
        // Add two sample region IDs (assuming RegionId is a type alias or newtype around u32 or similar)
        phi.add_region(RegionId::new(1), ControlFlowEdgeType::Branch);
        phi.add_region(RegionId::new(2), ControlFlowEdgeType::Fallthrough);
        assert_eq!(
            phi.regions(),
            &[
                (RegionId::new(1), ControlFlowEdgeType::Branch),
                (RegionId::new(2), ControlFlowEdgeType::Fallthrough)
            ]
        );
    }

    #[test]
    fn test_phi_node_equality() {
        let mut phi1 = PhiNode::new(0);
        phi1.add_region(RegionId::new(1), ControlFlowEdgeType::Branch);
        phi1.add_region(RegionId::new(2), ControlFlowEdgeType::Fallthrough);

        let mut phi2 = PhiNode::new(0);
        phi2.add_region(RegionId::new(1), ControlFlowEdgeType::Branch);
        phi2.add_region(RegionId::new(2), ControlFlowEdgeType::Fallthrough);

        assert_eq!(phi1, phi2);

        let mut phi3 = PhiNode::new(0);
        phi3.add_region(RegionId::new(1), ControlFlowEdgeType::Branch);
        phi3.add_region(RegionId::new(3), ControlFlowEdgeType::Fallthrough);

        assert_ne!(phi1, phi3);
    }
}
