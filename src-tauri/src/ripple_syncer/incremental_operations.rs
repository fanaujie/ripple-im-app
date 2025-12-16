use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum OpCategory {
    Add,
    Update,
    Delete,
}
pub trait Categorized {
    fn category(&self) -> OpCategory;
}

#[derive(Debug, Clone)]
pub struct Operation<T, O> {
    pub op: O,
    pub id: String,
    pub data: Option<T>,
}

#[derive(Debug, Clone)]
enum OriginalState {
    New,
    Existing,
}

#[derive(Debug, Clone)]
enum TrackingState<T> {
    New { data: T },
    Existing { data: T },
    Deleted { original: OriginalState },
}

#[derive(Debug, Clone)]
pub struct InsertItem<T> {
    pub id: String,
    pub data: T,
}

#[derive(Debug, Clone)]
pub struct UpdateItem<T> {
    pub id: String,
    pub data: T,
}

#[derive(Debug, Clone)]
pub struct DeleteItem {
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct ProcessedResult<T> {
    pub inserts: Vec<InsertItem<T>>,
    pub updates: Vec<UpdateItem<T>>,
    pub deletes: Vec<DeleteItem>,
}

pub fn process_incremental_operations<T, O>(operations: Vec<Operation<T, O>>) -> ProcessedResult<T>
where
    T: Clone,
    O: Categorized,
{
    let mut tracking: HashMap<String, TrackingState<T>> = HashMap::new();

    for op in operations {
        let id = op.id.clone();
        let category = op.op.category();
        match tracking.get(&id) {
            None => match category {
                OpCategory::Add => {
                    if let Some(data) = op.data {
                        tracking.insert(id, TrackingState::New { data });
                    }
                }
                OpCategory::Update => {
                    if let Some(data) = op.data {
                        tracking.insert(id, TrackingState::Existing { data });
                    }
                }
                OpCategory::Delete => {
                    tracking.insert(
                        id,
                        TrackingState::Deleted {
                            original: OriginalState::Existing,
                        },
                    );
                }
            },
            Some(current) => {
                // 用 category 取代原本的 op.op 判斷
                let new_state = match (category, current) {
                    (OpCategory::Add, TrackingState::Deleted { original }) => {
                        if let Some(data) = op.data {
                            match original {
                                OriginalState::New => Some(TrackingState::New { data }),
                                OriginalState::Existing => Some(TrackingState::Existing { data }),
                            }
                        } else {
                            None
                        }
                    }
                    (OpCategory::Add, _) => None,

                    (OpCategory::Update, TrackingState::New { .. }) => {
                        op.data.map(|data| TrackingState::New { data })
                    }
                    (OpCategory::Update, TrackingState::Existing { .. }) => {
                        op.data.map(|data| TrackingState::Existing { data })
                    }
                    (OpCategory::Update, TrackingState::Deleted { .. }) => None,

                    (OpCategory::Delete, TrackingState::New { .. }) => {
                        Some(TrackingState::Deleted {
                            original: OriginalState::New,
                        })
                    }
                    (OpCategory::Delete, TrackingState::Existing { .. }) => {
                        Some(TrackingState::Deleted {
                            original: OriginalState::Existing,
                        })
                    }
                    (OpCategory::Delete, TrackingState::Deleted { .. }) => None,
                };

                if let Some(state) = new_state {
                    tracking.insert(id, state);
                }
            }
        }
    }
    let mut inserts = Vec::new();
    let mut updates = Vec::new();
    let mut deletes = Vec::new();
    for (id, state) in tracking {
        match state {
            TrackingState::New { data } => {
                inserts.push(InsertItem { id, data });
            }
            TrackingState::Existing { data } => {
                updates.push(UpdateItem { id, data });
            }
            TrackingState::Deleted { .. } => {
                deletes.push(DeleteItem { id });
            }
        }
    }
    ProcessedResult {
        inserts,
        updates,
        deletes,
    }
}
