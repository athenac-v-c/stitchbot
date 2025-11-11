use petgraph::{Graph, Directed, graph::NodeIndex};
use kaspa_consensus_core::block::Block;
use std::collections::{HashMap, VecDeque};
use anyhow::Result;

pub type Dag = Graph<BlockInfo, (), Directed>;

#[derive(Clone, Debug)]
pub struct BlockInfo {
    pub hash: String,
    pub blue_score: u64,
    pub parents: Vec<String>,
    pub timestamp: u64,
}

pub struct RollingDag {
    graph: Dag,
    idx: HashMap<String, NodeIndex>,
    order: VecDeque<String>,
    capacity: usize,
}

impl RollingDag {
    pub fn new(capacity: usize) -> Self {
        Self {
            graph: Graph::new(),
            idx: HashMap::new(),
            order: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn add_block(&mut self, block: Block) -> bool {
        let info = BlockInfo {
            hash: block.hash().to_string(),
            blue_score: block.header.blue_score,
            parents: block.header.direct_parents.iter().map(|h| h.to_string()).collect(),
            timestamp: block.header.timestamp,
        };
        let hash = info.hash.clone();

        // Evict oldest
        if self.order.len() >= self.capacity {
            if let Some(old_hash) = self.order.pop_front() {
                if let Some(&node) = self.idx.get(&old_hash) {
                    self.graph.remove_node(node);
                }
                self.idx.remove(&old_hash);
            }
        }

        let node = self.graph.add_node(info.clone());
        self.idx.insert(hash.clone(), node);
        self.order.push_back(hash);

        for parent in &info.parents {
            if let Some(&p_node) = self.idx.get(parent) {
                self.graph.add_edge(p_node, node, ());
            }
        }
        true
    }

    pub fn find_fracture(&self, min_delta: u64) -> Option<(NodeIndex, Vec<NodeIndex>)> {
        use petgraph::algo::betweenness_centrality;
        let betweenness = betweenness_centrality(&self.graph);
        let mut candidates = vec![];

        for node in self.graph.node_indices() {
            let children: Vec<_> = self.graph.neighbors_directed(node, petgraph::Direction::Outgoing).collect();
            if children.len() < 2 { continue; }

            let info = &self.graph[node];
            let mut delta = u64::MAX;
            for &child in &children {
                let child_score = self.graph[child].blue_score;
                delta = delta.min(child_score.max(info.blue_score) - child_score.min(info.blue_score));
            }
            if delta < min_delta { continue; }

            candidates.push((node, betweenness[node.index()], delta));
        }

        candidates.sort_by_key(|&(i, bet, delta)| std::cmp::Reverse((bet * 1_000_000f64 + 1.0 / (delta as f64 + 1.0)) as u64));
        let best = candidates.first()?;
        let tips: Vec<_> = self.graph.neighbors_directed(best.0, petgraph::Direction::Outgoing).collect();
        Some((best.0, tips))
    }
}
