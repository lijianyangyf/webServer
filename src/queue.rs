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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_task_ordering() {
        let high_prio_task = Task {
            id: Uuid::new_v4(),
            payload: json!({}),
            priority: 100,
            retry_count: 0,
        };

        let low_prio_task = Task {
            id: Uuid::new_v4(),
            payload: json!({}),
            priority: 10,
            retry_count: 0,
        };

        assert!(high_prio_task > low_prio_task);
    }

    #[tokio::test]
    async fn test_priority_queue_push_pop() {
        let queue = PriorityQueue::new();

        let low_prio_task = Task {
            id: Uuid::new_v4(),
            payload: json!({ "task": "low" }),
            priority: 10,
            retry_count: 0,
        };
        let high_prio_task = Task {
            id: Uuid::new_v4(),
            payload: json!({ "task": "high" }),
            priority: 100,
            retry_count: 0,
        };

        queue.push(low_prio_task.clone()).await;
        queue.push(high_prio_task.clone()).await;

        let first_popped = queue.pop().await.unwrap();
        assert_eq!(first_popped.priority, high_prio_task.priority);

        let second_popped = queue.pop().await.unwrap();
        assert_eq!(second_popped.priority, low_prio_task.priority);

        assert!(queue.pop().await.is_none());
    }
}
