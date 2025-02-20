use androscalpel::{IdMethodType, Instruction};
use anyhow::{Context, Result};
use std::collections::HashMap;

struct Node<'a> {
    /// Code represented by the block
    code_block: &'a [Instruction],
    /// Label of the node if it exists
    label: Option<String>,
    /// Indices in CodeGraph.nodes of the next nodes
    next_nodes: Vec<usize>,
    /// Indices in CodeGraph.nodes of the previous nodes
    prev_nodes: Vec<usize>,
}

/// The CFG for a method, with potentially additionnal informations.
pub struct CFG<'a> {
    nodes: Vec<Node<'a>>,
}

impl<'a> CFG<'a> {
    pub fn new(_nb_reg: usize, _proto: &IdMethodType, insns: &'a [Instruction]) -> Result<Self> {
        let mut nodes = vec![Node {
            code_block: &insns[0..0],
            label: None,
            next_nodes: vec![],
            prev_nodes: vec![],
        }];
        let mut nodes_next_label = vec![vec![]];
        let nb_insns = insns.len();
        if nb_insns != 0 {
            nodes[0].next_nodes.push(1);
        }
        let mut start_last_block = 0;
        let mut last_label = None;
        let mut block_started = false;
        let mut try_block: Vec<(String, Vec<String>)> = vec![];
        for (i, ins) in insns.iter().enumerate() {
            match ins {
                // TODO: handle error better: list ins that can throw exceptions better
                Instruction::Throw { .. }
                | Instruction::InvokeVirtual { .. }
                | Instruction::InvokeSuper { .. }
                | Instruction::InvokeDirect { .. }
                | Instruction::InvokeDirect { .. }
                | Instruction::InvokeInterface { .. }
                | Instruction::InvokePolymorphic { .. }
                | Instruction::InvokeCustom { .. }
                    if !try_block.is_empty() =>
                {
                    nodes_next_label.push(try_block.last().unwrap().1.clone());
                    let next_nodes =
                        if i + 1 < nb_insns && !matches!(ins, Instruction::Throw { .. }) {
                            vec![nodes.len()+1] // If no exception, continue to next ins
                        } else {
                            vec![]
                        };
                    nodes.push(Node {
                        code_block: &insns[start_last_block..i + 1],
                        label: last_label,
                        next_nodes,
                        prev_nodes: vec![],
                    });
                    start_last_block = i + 1;
                    last_label = None;
                    block_started = false;
                }
                Instruction::Goto { label } => {
                    nodes_next_label.push(vec![label.clone()]);
                    nodes.push(Node {
                        code_block: &insns[start_last_block..i + 1],
                        label: last_label,
                        next_nodes: vec![], // Do not continue the execution at next ins
                        prev_nodes: vec![],
                    });
                    start_last_block = i + 1;
                    last_label = None;
                    block_started = false;
                }
                Instruction::Switch { branches, .. } => {
                    nodes_next_label.push(branches.values().cloned().collect());
                    let next_nodes = if i + 1 < nb_insns {
                        vec![nodes.len()+1] // If no branches match, continue execution
                    } else {
                        vec![]
                    };
                    nodes.push(Node {
                        code_block: &insns[start_last_block..i + 1],
                        label: last_label,
                        next_nodes,
                        prev_nodes: vec![],
                    });
                    start_last_block = i + 1;
                    last_label = None;
                    block_started = false;
                }
                Instruction::IfEq { label, .. }
                | Instruction::IfNe { label, .. }
                | Instruction::IfLt { label, .. }
                | Instruction::IfGe { label, .. }
                | Instruction::IfGt { label, .. }
                | Instruction::IfLe { label, .. }
                | Instruction::IfEqZ { label, .. }
                | Instruction::IfNeZ { label, .. }
                | Instruction::IfLtZ { label, .. }
                | Instruction::IfGeZ { label, .. }
                | Instruction::IfGtZ { label, .. }
                | Instruction::IfLeZ { label, .. } => {
                    nodes_next_label.push(vec![label.clone()]);
                    let next_nodes = if i + 1 < nb_insns {
                        vec![nodes.len()+1] // depending on test, continue execution
                    } else {
                        vec![]
                    };
                    nodes.push(Node {
                        code_block: &insns[start_last_block..i + 1],
                        label: last_label,
                        next_nodes,
                        prev_nodes: vec![],
                    });
                    start_last_block = i + 1;
                    last_label = None;
                    block_started = false;
                }
                Instruction::Try {
                    end_label,
                    handlers,
                    default_handler,
                } => {
                    let mut branches: Vec<_> =
                        handlers.iter().map(|(_, label)| label.clone()).collect();
                    if let Some(default_handler) = default_handler.as_ref().cloned() {
                        branches.push(default_handler);
                    }
                    try_block.push((end_label.clone(), branches))
                }
                Instruction::Label { name } => {
                    if !block_started {
                        last_label = Some(name.clone());
                    } else {
                        nodes_next_label.push(vec![]);
                        nodes.push(Node {
                            code_block: &insns[start_last_block..i],
                            label: last_label,
                            next_nodes: vec![nodes.len()+1],
                            prev_nodes: vec![],
                        });
                        start_last_block = i;
                        last_label = Some(name.clone());
                    }
                }
                Instruction::ReturnVoid {}
                | Instruction::Return { .. }
                | Instruction::ReturnWide { .. }
                | Instruction::ReturnObject { .. }
                | Instruction::Throw { .. } => {
                    nodes_next_label.push(vec![]);
                    nodes.push(Node {
                        code_block: &insns[start_last_block..i + 1],
                        label: last_label,
                        next_nodes: vec![], // Do not continue the execution at next ins
                        prev_nodes: vec![],
                    });
                    start_last_block = i + 1;
                    last_label = None;
                    block_started = false;
                }
                _ => if !ins.is_pseudo_ins() { block_started = true; },
            }
        }
        let label_to_node: HashMap<String, usize> = nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| node.label.is_some())
            .map(|(i, node)| (node.label.as_ref().unwrap().clone(), i))
            .collect();
        for (node, labels) in nodes.iter_mut().zip(nodes_next_label) {
            for label in labels {
                node.next_nodes
                    .push(*label_to_node.get(&label).with_context(|| {
                        format!("found jumb to label '{}' but label not found", label)
                    })?);
            }
        }

        for i in 0..nodes.len() {
            let next_nodes = nodes[i].next_nodes.clone();
            for j in &next_nodes {
                nodes[*j].prev_nodes.push(i);
            }
        }
        Ok(Self { nodes })
    }

    /// Serialize the graph to dot format.
    pub fn to_dot(&self, name: &str) -> String {
        let mut dot_string = "digraph {\n".to_string();
        dot_string += "    overlap=false;\n";
        dot_string += "    style=\"dashed\";\n";
        dot_string += "    color=\"black\";\n";
        dot_string += &format!("    label=\"{name}\";\n");
        for (i, node) in self.nodes.iter().enumerate() {
            let block_name = if i == 0 {
                "ENTRY".into()
            } else if let Some(label) = node.label.as_ref() {
                format!("block '{label}'")
            } else {
                format!("block {i}")
            };
            let label = if node.code_block.is_empty() {
                format!("{{\\< {block_name} \\>}}")
            } else {
                let mut label = format!("{{\\< {block_name} \\>:\\l\\\n");
                for ins in node.code_block {
                    label += "|";
                    label += ins
                        .__str__()
                        .replace(" ", "\\ ")
                        .replace(">", "\\>")
                        .replace("<", "\\<")
                        .replace("\"", "\\\"")
                        .replace("{", "\\{")
                        .replace("}", "\\}")
                        .as_str();
                    label += "\\l\\\n";
                }
                label += "}";
                label
            };
            dot_string += &format!(
                "    node_{i} [shape=record,style=filled,fillcolor=lightgrey,label=\"{label}\"];\n\n"
            );
        }
        dot_string += 
                "    node_end [shape=record,style=filled,fillcolor=lightgrey,label=\"{\\< EXIT \\>}\"];\n\n";

        for (i, node) in self.nodes.iter().enumerate() {
            for j in &node.next_nodes {
                if *j == i + 1 {
                    dot_string += &format!("    node_{i}:s -> node_{j}:n [style=\"solid,bold\",color=black,weight=100,constraint=true];\n");
                } else {
                    dot_string += &format!("    node_{i}:s -> node_{j}:n [style=\"solid,bold\",color=black,weight=10,constraint=true];\n");
                }
            }
            if node.next_nodes.is_empty() {
                    dot_string += &format!("    node_{i}:s -> node_end:n [style=\"solid,bold\",color=black,weight=10,constraint=true];\n");
            }
        }
        dot_string += "}\n";
        dot_string
    }
}
