# Time-Partitioned Scheduler

A complete implementation of **GreenHills Integrity RTOS**-style time and space partitioned scheduling for WasmFlow.

## 🎯 What Is This?

An educational scheduler that demonstrates real-time operating system (RTOS) concepts by executing multiple WASM components with deterministic timing guarantees. Think of it as a mini Integrity RTOS running inside a visual node graph!

### Key Concepts

- **Space Partitioning**: Each task runs in its own isolated WASM component (memory isolation)
- **Time Partitioning**: Strict time budgets enforced via timeouts (temporal isolation)
- **Deterministic Scheduling**: 5 scheduling algorithms (Round-Robin, Priority, RMS, EDF, LLF)
- **Visual Teaching**: Real-time Gantt chart shows which tasks execute when

## 🚀 Quick Start

### 1. Load an Example Graph

**Priority-Based Scheduling:**
```
File → Open → examples/scheduler_demo.json
```
Demonstrates 3 math tasks with different priorities (200, 150, 100).

**Periodic Real-Time Tasks:**
```
File → Open → examples/scheduler_periodic_demo.json
```
Simulates a real-time system with tasks at 10Hz, 2Hz, and 0.2Hz using EDF.

### 2. Start the Scheduler

1. Find the blue scheduler node (labeled "Time-Partitioned Scheduler")
2. Right-click on it
3. Select **"Start"**
4. The scheduler begins executing immediately!

### 3. Watch the Visualization

Click on the scheduler node to see the footer, which displays:
- **Statistics**: Iterations, context switches, CPU utilization, violations
- **Gantt Chart**: Real-time timeline showing which task ran when
- **Color Coding**: Green (success), Orange (budget exceeded), Red (deadline missed)

## 📚 Documentation

| Document | Description |
|----------|-------------|
| **QUICK_START.md** | Simple guide to using JSON input and basic configuration |
| **SCHEDULER_USAGE.md** | Comprehensive documentation of all features and algorithms |
| **GANTT_CHART.md** | Guide to the visualization and teaching exercises |
| **IMPLEMENTATION_SUMMARY.md** | Technical details of how it's built |
| **examples/README_SCHEDULER.md** | Complete guide to example graphs |

## 🎓 Educational Features

### Visual Feedback

The Gantt chart makes abstract RTOS concepts **observable**:

```
High Priority Add       ████░░░░░░░░░░░░░░░░  (4ms - Success)
Medium Priority Mult    ░░░░████████░░░░░░░░  (6ms - Success)
Low Priority Divide     ░░░░░░░░░░░░████░░░░  (5ms - Budget Exceeded!)
```

- See priority scheduling in action
- Watch budget violations occur (orange bars)
- Observe deadline misses (red bars)
- Monitor CPU utilization and context switches

### Learning Exercises

**Built-in teaching scenarios** (see GANTT_CHART.md):
1. **Priority Inversion**: Change all priorities to same value, observe round-robin
2. **Budget Overruns**: Set budgets to 10ms, watch violations
3. **Deadline Misses**: Increase budgets, see deadline monitoring
4. **CPU Saturation**: Add more tasks, watch utilization climb to 100%

## 🔧 How to Use

### The Easy Way: JSON Input

Create a **String Constant** with your task list:

```json
[
  {
    "component_id": "user:math-adder",
    "priority": 200,
    "budget_ms": 50,
    "display_name": "High Priority Task"
  },
  {
    "component_id": "user:echo",
    "priority": 100,
    "budget_ms": 75,
    "period_ms": 1000,
    "deadline_ms": 900,
    "display_name": "Periodic Task"
  }
]
```

Connect it to the scheduler's `tasks_json` input and start!

### Configuration Inputs

| Input | Type | Description | Default |
|-------|------|-------------|---------|
| `tasks_json` | String | JSON array of task configs | Required |
| `schedule_mode` | String | Algorithm: "round-robin", "priority-based", "rate-monotonic", "earliest-deadline-first", "least-laxity-first" | "round-robin" |
| `time_quantum_ms` | U32 | Time slice for round-robin (ms) | 100 |
| `enable_preemption` | Bool | Allow task preemption | false |

### Output Ports

| Output | Type | Description |
|--------|------|-------------|
| `status` | String | "executed", "idle", "no_tasks" |
| `executed_task` | String | Name of task that just ran |
| `execution_time_ms` | U32 | Actual execution duration |
| `budget_exceeded` | Bool | Budget violation indicator |
| `execution_success` | Bool | Task completion status |
| `schedule_state` | Record | **Complete statistics + history for Gantt chart** |

### Task Configuration Fields

**Required:**
- `component_id` - Component to execute (e.g., "user:math-adder")

**Optional:**
- `priority` (0-255) - Higher = more important [default: 128]
- `budget_ms` - Max execution time in ms [default: 100]
- `period_ms` - Period for periodic tasks, 0 = aperiodic [default: 0]
- `deadline_ms` - Relative deadline, 0 = use period [default: 0]
- `display_name` - Name shown in visualization [default: component_id]

## 🧠 Scheduling Algorithms

### 1. Round-Robin (Default)
Fair time-slicing. Each task gets equal time quantum.
```
Task order: A → B → C → A → B → C → ...
```

### 2. Priority-Based
Static priority scheduling. High-priority tasks always execute first.
```
Priority 200 (High) runs most often
Priority 150 (Medium) runs occasionally
Priority 100 (Low) runs rarely
```

### 3. Rate Monotonic Scheduling (RMS)
Shorter period = higher priority. Optimal for periodic tasks.
```
100ms period → Highest priority
500ms period → Medium priority
5000ms period → Lowest priority
```

### 4. Earliest Deadline First (EDF)
Dynamic priority by absolute deadline. Best for meeting deadlines.
```
Task with nearest deadline executes next
Priorities change based on deadlines
```

### 5. Least Laxity First (LLF)
Minimum slack time first. Laxity = deadline - remaining_time - execution_time.
```
Task with least "wiggle room" executes next
Prevents last-minute deadline misses
```

## 📊 What You'll Learn

### RTOS Fundamentals

- **Space Partitioning**: WASM component isolation = memory protection domains
- **Time Partitioning**: Timeout enforcement = temporal isolation guarantees
- **Deterministic Scheduling**: Predictable, analyzable execution behavior
- **Safety Monitoring**: Budget/deadline violation detection
- **Context Switching**: Overhead of task switches tracked

### Real-Time Systems

- **Periodic Tasks**: Jobs that run at regular intervals (sensors, actuators)
- **Aperiodic Tasks**: Jobs triggered by events
- **Deadline Monitoring**: Ensuring time-critical tasks complete on time
- **Schedulability Analysis**: CPU utilization limits
- **Priority Assignment**: How to choose task priorities

### Integrity RTOS Concepts

This scheduler recreates the **two key partitioning mechanisms** from GreenHills Integrity:

1. **Space Partitioning** (via WASM):
   - Each task = separate WASM component
   - No shared memory between tasks
   - True isolation like Integrity memory domains

2. **Time Partitioning** (via timeouts):
   - Each task has fixed time budget
   - Budget exceeded → task forcibly terminated
   - Prevents tasks from stealing CPU time

## 🎯 Use Cases

### 1. Teaching RTOS Concepts

Use in embedded systems or real-time systems courses to:
- Visualize scheduling algorithms
- Demonstrate priority inversion
- Show budget enforcement
- Explore deadline monitoring

### 2. Prototyping Real-Time Systems

Before coding on hardware:
- Test scheduling strategies
- Measure CPU utilization
- Validate task timing budgets
- Detect potential deadline misses

### 3. Understanding Deterministic Execution

See how different algorithms affect:
- Response times
- Context switch overhead
- CPU utilization
- Deadline guarantees

## 🔍 Example Scenarios

### Scenario 1: High-Speed Data Acquisition

```json
[
  {
    "component_id": "user:adc-read",
    "priority": 200,
    "budget_ms": 15,
    "period_ms": 100,
    "deadline_ms": 90,
    "display_name": "ADC Sampling (10Hz)"
  },
  {
    "component_id": "user:data-process",
    "priority": 150,
    "budget_ms": 50,
    "period_ms": 500,
    "deadline_ms": 450,
    "display_name": "Signal Processing (2Hz)"
  },
  {
    "component_id": "user:data-logger",
    "priority": 50,
    "budget_ms": 100,
    "display_name": "Background Logger"
  }
]
```

**Use EDF algorithm** for deadline guarantees. Watch Gantt chart to verify all deadlines are met.

### Scenario 2: Control System

```json
[
  {
    "component_id": "user:sensor-read",
    "priority": 255,
    "budget_ms": 20,
    "display_name": "Critical Sensor"
  },
  {
    "component_id": "user:pid-controller",
    "priority": 200,
    "budget_ms": 40,
    "display_name": "PID Loop"
  },
  {
    "component_id": "user:actuator-update",
    "priority": 180,
    "budget_ms": 30,
    "display_name": "Motor Control"
  },
  {
    "component_id": "user:telemetry",
    "priority": 50,
    "budget_ms": 100,
    "display_name": "Telemetry"
  }
]
```

**Use Priority-Based algorithm**. Sensor → Controller → Actuator loop gets priority over telemetry.

## 🛠️ Technical Details

### Implementation

- **Language**: Rust
- **Location**: `src/builtin/scheduler/`
- **Type**: Builtin continuous node (not WASM component)
- **Execution**: Async via tokio runtime
- **State**: Persistent across iterations via static cache

### Performance

- **Scheduler overhead**: ~1-5ms per iteration
- **Timeout precision**: ~1-5ms (tokio granularity)
- **Memory**: Scales linearly with task count
- **Visualization**: <10ms render time (imperceptible)

### Architecture

```
┌─────────────────────────────────────┐
│   Scheduler Builtin Node            │
│                                     │
│  ┌─────────────────────────────┐   │
│  │  Executor (executor.rs)     │   │
│  │  - Continuous execution     │   │
│  │  - Component invocation     │   │
│  │  - Timeout enforcement      │   │
│  │  - Statistics tracking      │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │  Algorithms (algorithms.rs) │   │
│  │  - RR, Priority, RMS, EDF   │   │
│  │  - LLF                      │   │
│  └─────────────────────────────┘   │
│                                     │
│  ┌─────────────────────────────┐   │
│  │  Views (views.rs)           │   │
│  │  - Gantt chart rendering    │   │
│  │  - Statistics dashboard     │   │
│  └─────────────────────────────┘   │
└─────────────────────────────────────┘
```

### Files

- `src/builtin/scheduler/mod.rs` - Module declaration
- `src/builtin/scheduler/executor.rs` - Main scheduler logic
- `src/builtin/scheduler/algorithms.rs` - Scheduling algorithms
- `src/builtin/scheduler/timing.rs` - Timing utilities
- `src/builtin/scheduler/config.rs` - Data structures
- `src/builtin/scheduler/views.rs` - Gantt chart visualization

## 🚦 Getting Started Checklist

- [ ] Load an example graph (`examples/scheduler_demo.json`)
- [ ] Start the scheduler (right-click → Start)
- [ ] Click node to view Gantt chart footer
- [ ] Watch tasks execute in real-time
- [ ] Try modifying task priorities in JSON
- [ ] Experiment with different scheduling algorithms
- [ ] Create budget violations (set budget_ms to 10)
- [ ] Add more tasks to increase CPU utilization
- [ ] Try periodic tasks with EDF algorithm
- [ ] Read GANTT_CHART.md for teaching exercises

## 📖 Further Reading

- **GreenHills Integrity RTOS**: [integrity.com](https://www.ghs.com/products/rtos/integrity.html)
- **RTOS Scheduling**: [wikipedia.org/wiki/Rate-monotonic_scheduling](https://en.wikipedia.org/wiki/Rate-monotonic_scheduling)
- **WasmFlow Documentation**: `docs/` directory
- **Component Development**: `components/LIBRARY.md`

## 🎉 Credits

**Implementation**: WasmFlow Scheduler
**Inspired by**: GreenHills Integrity RTOS
**Date**: 2025-11-09
**Version**: 1.0.0

---

**Enjoy teaching RTOS concepts with visual, interactive demonstrations!** 🚀
