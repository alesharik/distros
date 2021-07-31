use alloc::sync::Arc;
use spin::Mutex;
use crate::flow::{Sender, Provider, Message};
use alloc::string::String;
use hashbrown::HashMap;
use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use core::any::TypeId;
use core::fmt::{Formatter, Display};

pub struct FlowTreeEndpoint {
    pub provider: Arc<Mutex<dyn Provider + Send>>,
    pub sender: Option<Arc<Mutex<dyn Sender<Msg = dyn Message> + Send>>>,
    pub message_type: TypeId,
}

impl Clone for FlowTreeEndpoint {
    fn clone(&self) -> Self {
        FlowTreeEndpoint {
            sender: self.sender.clone(),
            provider: self.provider.clone(),
            message_type: self.message_type.clone(),
        }
    }
}

impl FlowTreeEndpoint {
    pub fn new<T: Message + 'static>(provider: Arc<Mutex<dyn Provider + Send>>) -> Self {
        FlowTreeEndpoint {
            provider,
            sender: None,
            message_type: TypeId::of::<T>(),
        }
    }

    pub fn sender<'a>(&'a mut self, sender: Arc<Mutex<dyn Sender<Msg = dyn Message> + Send>>) -> &'a mut FlowTreeEndpoint {
        self.sender = Some(sender);
        self
    }
}

pub enum FlowTreeError {
    AlreadyOccupied,
    WrongMessageType,
}

type Result<T> = core::result::Result<T, FlowTreeError>;

#[derive(Default)]
struct FlowTreeBranch {
    nodes: HashMap<String, FlowTreeNode>,
    this_endpoint: Option<FlowTreeEndpoint>
}

enum FlowTreeNode {
    Endpoint(FlowTreeEndpoint),
    Branch(FlowTreeBranch),
    // Collector() TODO
}

pub struct ElementInfo {
    pub name: String,
    pub directory: bool,
}

impl Display for ElementInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.name)?;
        if self.directory {
            write!(f, "/")?;
        }
        Ok(())
    }
}

pub struct FlowTree {
    node: FlowTreeBranch,
}

impl FlowTree {
    pub fn new() -> Self {
        FlowTree {
            node: FlowTreeBranch {
                nodes: HashMap::new(),
                this_endpoint: None
            },
        }
    }

    pub fn put<T: Message + 'static>(&mut self, path: &str, item: FlowTreeEndpoint) -> Result<()> {
        let parts = path.split("/")
            .collect::<Vec<_>>();
        let (last, parts) = parts
            .split_last()
            .unwrap();
        let mut current = &mut self.node;
        for part in parts {
            let part = part.to_owned().to_owned();
            let node = current.nodes.entry(part)
                .and_replace_entry_with(|_k, entry| Some(FlowTreeNode::Branch(match entry {
                    FlowTreeNode::Branch(b) => b,
                    FlowTreeNode::Endpoint(endpoint) => {
                        let mut new_branch = FlowTreeBranch::default();
                        new_branch.this_endpoint = Some(endpoint);
                        new_branch
                    }
                })))
                .or_insert_with(|| FlowTreeNode::Branch(FlowTreeBranch::default()));
            current = if let FlowTreeNode::Branch(b) = node { b } else { unreachable!() };
        }

        if let Err(mut error) = current.nodes.try_insert(last.to_owned().to_owned(), FlowTreeNode::Endpoint(item.clone())) {
            let placed = error.entry.get_mut();
            match placed {
                FlowTreeNode::Endpoint(_) => Err(FlowTreeError::AlreadyOccupied),
                FlowTreeNode::Branch(branch) => if branch.this_endpoint.is_some() {
                    Err(FlowTreeError::AlreadyOccupied)
                } else {
                    branch.this_endpoint = Some(item);
                    Ok(())
                }
            }
        } else {
            Ok(())
        }
    }

    pub fn get(&self, path: &str) -> Option<FlowTreeEndpoint> {
        let parts = path.split("/")
            .collect::<Vec<_>>();
        let (last, parts) = parts
            .split_last()
            .unwrap();
        let mut current = &self.node;
        for part in parts {
            let part = part.to_owned();
            match current.nodes.get(part)? {
                FlowTreeNode::Branch(branch) => current = branch,
                FlowTreeNode::Endpoint(_) => return None
            }
        }
        let last = last.to_owned();
        match current.nodes.get(last)? {
            FlowTreeNode::Branch(branch) => branch.this_endpoint.clone(),
            FlowTreeNode::Endpoint(endpoint) => Some(endpoint.clone())
        }
    }

    pub fn list(&self, path: &str) -> Vec<ElementInfo> {
        let parts = path.split("/")
            .collect::<Vec<_>>();
        let (last, parts) = parts
            .split_last()
            .unwrap();
        let mut current = &self.node;
        for part in parts {
            let part = part.to_owned();
            match current.nodes.get(part) {
                Some(node) => match node {
                    FlowTreeNode::Branch(branch) => current = branch,
                    FlowTreeNode::Endpoint(_) => return vec![]
                }
                None => return vec![]
            }
        }
        let last = last.to_owned();
        if last == "" {
            let nodes = current.nodes.iter().map(|(k, v)| ElementInfo {
                name: k.clone(),
                directory: matches!(v, FlowTreeNode::Branch(_))
            }).collect::<Vec<_>>();
            return nodes
        }
        match current.nodes.get(last) {
            Some(node) => match node {
                FlowTreeNode::Branch(branch) => {
                    let mut nodes = branch.nodes.iter().map(|(k, v)| ElementInfo {
                        name: k.clone(),
                        directory: matches!(v, FlowTreeNode::Branch(_))
                    }).collect::<Vec<_>>();
                    if branch.this_endpoint.is_some() {
                        nodes.push(ElementInfo { name: last.to_owned(), directory: false })
                    }
                    nodes
                },
                FlowTreeNode::Endpoint(_) => vec![ElementInfo {
                    name: last.to_owned(),
                    directory: false
                }]
            }
            None => vec![]
        }
    }
}