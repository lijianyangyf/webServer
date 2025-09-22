use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BinaryHeap;
use std::cmp::Ordering;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: Uuid,
    pub payload: Value,
    pub priority: u8,
    pub retry_count: u8,
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

impl Eq for Task {}

impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

pub struct PriorityQueue {
    heap: Mutex<BinaryHeap<Task>>,
}

impl PriorityQueue {
    pub fn new() -> Self {
        Self { heap: Mutex::new(BinaryHeap::new()) }
    }

    pub async fn push(&self, task: Task) {
        let mut heap = self.heap.lock().await;
        heap.push(task);
    }

    pub async fn pop(&self) -> Option<Task> {
        let mut heap = self.heap.lock().await;
        heap.pop()
    }
}
