# Scheduler Quick Start - Simplified!

## 🎯 The Easy Way (JSON Input)

No complex component chains needed! Just paste JSON directly.

### Step 1: Create a Task List JSON

Create a **String Constant** node with your task list:

```json
[
  {
    "component_id": "user:math-adder",
    "priority": 150,
    "budget_ms": 50,
    "display_name": "Add Numbers"
  },
  {
    "component_id": "user:double-number",
    "priority": 100,
    "budget_ms": 75,
    "display_name": "Double Result"
  },
  {
    "component_id": "user:string-concat",
    "priority": 80,
    "budget_ms": 60,
    "period_ms": 1000,
    "deadline_ms": 900,
    "display_name": "String Task"
  }
]
```

### Step 2: Connect to Scheduler

1. Add "Time-Partitioned Scheduler" node (Scheduler category)
2. Connect your String Constant to the `tasks_json` input
3. Right-click scheduler → **Start**
4. Watch it execute your tasks!

### Step 3: Monitor Execution

Check the output ports:
- `status` - Current state: "executed", "idle", or "no_tasks"
- `executed_task` - Name of task that just ran
- `execution_time_ms` - How long it took
- `budget_exceeded` - Did it exceed the budget?
- `schedule_state` - Full statistics (CPU utilization, violations, etc.)

## 📋 Task Configuration Fields

### Required
- `component_id` (string) - Component to execute (e.g., "user:math-adder")

### Optional (with defaults)
- `priority` (number, 0-255) - Higher = more important [default: 128]
- `budget_ms` (number) - Max execution time in ms [default: 100]
- `period_ms` (number) - Period for periodic tasks, 0 = aperiodic [default: 0]
- `deadline_ms` (number) - Relative deadline, 0 = use period [default: 0]
- `display_name` (string) - Name shown in visualization [default: component_id]
- `inputs` (object) - Inputs to pass to component [default: {}]

## 🎓 Example: Round-Robin (Equal Priority)

All tasks get equal time slices:

```json
[
  {"component_id": "user:math-adder", "priority": 100, "budget_ms": 50},
  {"component_id": "user:multiplier", "priority": 100, "budget_ms": 50},
  {"component_id": "user:divider", "priority": 100, "budget_ms": 50}
]
```

Result: 1 → 2 → 3 → 1 → 2 → 3 → ...

## 🎓 Example: Priority-Based Scheduling

High priority tasks run first:

```json
[
  {"component_id": "user:sensor-reader", "priority": 200, "budget_ms": 30, "display_name": "Critical Sensor"},
  {"component_id": "user:data-processor", "priority": 150, "budget_ms": 50, "display_name": "Data Processing"},
  {"component_id": "user:logger", "priority": 50, "budget_ms": 100, "display_name": "Background Logger"}
]
```

Result: Critical Sensor always runs first!

## 🎓 Example: Periodic Tasks (Real-Time System)

Tasks run at specific intervals:

```json
[
  {
    "component_id": "user:sensor-reader",
    "priority": 200,
    "budget_ms": 20,
    "period_ms": 100,
    "deadline_ms": 90,
    "display_name": "Fast Sensor (10Hz)"
  },
  {
    "component_id": "user:actuator-update",
    "priority": 180,
    "budget_ms": 40,
    "period_ms": 500,
    "deadline_ms": 450,
    "display_name": "Actuator (2Hz)"
  },
  {
    "component_id": "user:status-report",
    "priority": 50,
    "budget_ms": 100,
    "period_ms": 5000,
    "deadline_ms": 4900,
    "display_name": "Status (0.2Hz)"
  }
]
```

The scheduler ensures all deadlines are met!

## ⚙️ Scheduler Configuration (Optional)

Add these constants to the scheduler's other inputs:

### `time_quantum_ms` (U32 Constant)
Time slice for round-robin mode. Default: 100ms

### `schedule_mode` (String Constant)
Choose scheduling algorithm:
- `"round-robin"` - Equal time slices (default)
- `"priority-based"` - Highest priority first
- `"rate-monotonic"` - Shorter period = higher priority
- `"earliest-deadline-first"` - Dynamic priority by deadline
- `"least-laxity-first"` - Minimum slack time first

### `enable_preemption` (Bool Constant)
Allow high-priority tasks to interrupt low-priority ones. Default: false

## 🎯 Complete Example Graph

```
[String Constant]              [U32 Constant]      [String Constant]
  JSON task list     ─────────>  100ms       ──────>  "priority-based"
       │                           │                      │
       │                           ▼                      ▼
       └──────────────>  [Time-Partitioned Scheduler]
                              (continuous node)
                                    │
                                    ├──> status
                                    ├──> executed_task
                                    ├──> execution_time_ms
                                    ├──> budget_exceeded
                                    └──> schedule_state
```

## 🚀 Next Steps

1. **Try it!** Copy the JSON examples above
2. **Monitor outputs** - Watch which tasks execute when
3. **Change algorithms** - See how behavior changes
4. **Add more tasks** - Scale up to 10+ tasks
5. **Set periods** - Create a real-time system simulation

## 💡 Pro Tips

- Start with 2-3 simple tasks to understand the behavior
- Use different priorities to see preemption in action
- Set budgets lower than expected execution time to see violations
- Use periodic tasks to simulate real-time systems
- Check `budget_exceeded` to catch time overruns
- Monitor `schedule_state` for detailed statistics

## 🐛 Troubleshooting

**"Failed to parse tasks_json"**
- Check your JSON syntax (use a validator)
- Make sure it's an array: `[{...}, {...}]`
- Component IDs must be strings in quotes

**"Component execution failed"**
- Verify component_id is correct (e.g., "user:math-adder")
- Check component is in `components/bin/`
- Look at `execution_error` output for details

**No tasks executing**
- Make sure you connected to `tasks_json` (not `tasks`)
- Check you **started** the continuous node (right-click → Start)
- Verify JSON has at least one task

**Budget exceeded frequently**
- Increase `budget_ms` values
- Or optimize the components being executed
- This is normal for teaching - shows budget enforcement!
