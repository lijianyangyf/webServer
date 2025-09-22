use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use tokio::sync::Mutex;
use uuid::Uuid;

/// 表示一个待处理的任务。
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    /// 任务的唯一标识符。
    pub id: Uuid,
    /// 任务的有效载荷，可以是任意 JSON 数据。
    pub payload: Value,
    /// 任务的优先级，数值越大，优先级越高。
    pub priority: u8,
    /// 任务的重试次数。
    pub retry_count: u8,
}

// 为 `Task` 实现 `PartialEq` trait，以便能够比较两个任务是否相等。
// 在这里，我们仅基于 `priority` 进行比较，这对于 `BinaryHeap` 的行为是足够的。
impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority
    }
}

// `Eq` 是一个标记 trait，表示 `eq` 方法实现了一个等价关系。
impl Eq for Task {}

// 为 `Task` 实现 `PartialOrd` trait，以定义任务之间的部分排序。
impl PartialOrd for Task {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// 为 `Task` 实现 `Ord` trait，以定义任务之间的全序关系。
// `BinaryHeap` 使用这个实现来确定元素的顺序，从而实现最大堆（优先级最高的在顶部）。
impl Ord for Task {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

/// 一个线程安全的异步优先级队列。
/// 内部使用 `tokio::sync::Mutex` 包裹的 `std::collections::BinaryHeap` 实现。
pub struct PriorityQueue {
    heap: Mutex<BinaryHeap<Task>>,
}

impl PriorityQueue {
    /// 创建一个新的空优先级队列。
    pub fn new() -> Self {
        Self {
            heap: Mutex::new(BinaryHeap::new()),
        }
    }

    /// 将一个任务异步推入队列。
    pub async fn push(&self, task: Task) {
        let mut heap = self.heap.lock().await;
        heap.push(task);
    }

    /// 从队列中异步弹出一个任务。
    /// 如果队列为空，则返回 `None`。
    /// 由于内部是最大堆，弹出的总是优先级最高的任务。
    pub async fn pop(&self) -> Option<Task> {
        let mut heap = self.heap.lock().await;
        heap.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// 测试 `Task` 的排序是否符合预期（基于优先级）。
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

    /// 测试 `PriorityQueue` 的 `push` 和 `pop` 操作是否正确。
    /// 应该先弹出优先级高的任务。
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

        // 第一次弹出的应该是高优先级的任务
        let first_popped = queue.pop().await.unwrap();
        assert_eq!(first_popped.priority, high_prio_task.priority);

        // 第二次弹出的应该是低优先级的任务
        let second_popped = queue.pop().await.unwrap();
        assert_eq!(second_popped.priority, low_prio_task.priority);

        // 队列现在应该为空
        assert!(queue.pop().await.is_none());
    }
}
