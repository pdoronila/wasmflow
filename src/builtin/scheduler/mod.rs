//! Time and Space Partitioned Scheduler
//!
//! Implements a scheduler inspired by GreenHills Integrity RTOS that demonstrates:
//! - **Space Partitioning**: Each task runs in isolated WASM component (memory isolation)
//! - **Time Partitioning**: Deterministic time budgets and scheduling algorithms
//! - **Visual Teaching**: Real-time Gantt chart visualization of schedule execution
//!
//! This is designed as a teaching tool to make abstract RTOS concepts observable.

pub mod algorithms;
pub mod config;
pub mod executor;
pub mod timing;
pub mod views;

pub use executor::{register_scheduler, TimePartitionedSchedulerExecutor};
pub use views::SchedulerFooterView;
