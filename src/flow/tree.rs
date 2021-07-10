use alloc::sync::Arc;
use spin::Mutex;
use crate::flow::{Sender, Provider, Message};
use alloc::string::String;
use hashbrown::HashMap;
use alloc::borrow::ToOwned;
use alloc::vec::Vec;
use alloc::boxed::Box;
use core::any::Any;
use core::fmt::{Debug, Formatter, Display};

pub struct FlowTreeEndpoint<T: Message> {
    pub provider: Arc<Mutex<dyn Provider<T> + Send>>,
    pub sender: Option<Arc<Mutex<dyn Sender<T> + Send>>>
}

impl<T: Message> Clone for FlowTreeEndpoint<T> {
    fn clone(&self) -> Self {
        FlowTreeEndpoint {
            sender: self.sender.clone(),
            provider: self.provider.clone(),
        }
    }
}

impl<T: Message> FlowTreeEndpoint<T> {
    pub fn new(provider: Arc<Mutex<dyn Provider<T> + Send>>) -> Self {
        FlowTreeEndpoint {
            provider,
            sender: None
        }
    }

    pub fn sender<'a>(&'a mut self, sender: Arc<Mutex<dyn Sender<T> + Send>>) -> &'a mut FlowTreeEndpoint<T> {
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
    this_endpoint: Option<Box<dyn Any + Send>>
}

enum FlowTreeNode {
    Endpoint(Box<dyn Any + Send>),
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

    pub fn put<T: Message + 'static>(&mut self, path: &str, item: FlowTreeEndpoint<T>) -> Result<()> {
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

        if let Err(mut error) = current.nodes.try_insert(last.to_owned().to_owned(), FlowTreeNode::Endpoint(Box::new(item.clone()))) {
            let placed = error.entry.get_mut();
            match placed {
                FlowTreeNode::Endpoint(_) => Err(FlowTreeError::AlreadyOccupied),
                FlowTreeNode::Branch(branch) => if branch.this_endpoint.is_some() {
                    Err(FlowTreeError::AlreadyOccupied)
                } else {
                    branch.this_endpoint = Some(Box::new(item));
                    Ok(())
                }
            }
        } else {
            Ok(())
        }
    }

    pub fn get<T: Message + 'static>(&self, path: &str) -> Result<Option<FlowTreeEndpoint<T>>> {
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
                    FlowTreeNode::Endpoint(_) => return Ok(None)
                }
                None => return Ok(None)
            }
        }
        let last = last.to_owned();
        match current.nodes.get(last) {
            Some(node) => match node {
                FlowTreeNode::Branch(branch) => match &branch.this_endpoint {
                    Some(e) => match e.downcast_ref::<FlowTreeEndpoint<T>>() {
                        Some(e) => Ok(Some(e.clone())),
                        None => Err(FlowTreeError::WrongMessageType)
                    },
                    None => Ok(None)
                },
                FlowTreeNode::Endpoint(endpoint) => match endpoint.downcast_ref::<FlowTreeEndpoint<T>>() {
                    Some(e) => Ok(Some(e.clone())),
                    None => Err(FlowTreeError::WrongMessageType)
                }
            }
            None => Ok(None)
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
            let mut nodes = current.nodes.iter().map(|(k, v)| ElementInfo {
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
                FlowTreeNode::Endpoint(endpoint) => vec![ElementInfo {
                    name: last.to_owned(),
                    directory: false
                }]
            }
            None => vec![]
        }
    }
}