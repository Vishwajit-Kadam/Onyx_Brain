# Brain Principles

Onyx Brain is brain-inspired, not a biological brain simulation.

## Sparse Activation

The whole system exists as potential, but only a small useful working set activates for a task. This keeps runtime state small and inspectable.

## Memory Hierarchy

The runtime separates different memory roles:

- semantic memory for facts
- procedural memory for reusable workflows
- episodic memory for task history
- project memory for sandbox project state

## Plasticity-Like Route Updates

Successful routes can be strengthened. Failed or expensive routes can become less attractive. This is an engineering analogy to plasticity, not a neuroscience model.

## Habit Formation

Repeated successful workflows can become habits. A habit is a compressed plan template with confidence and trigger patterns. Habits can reduce planning overhead but must not bypass validation.

## Prediction And Action Loops

The runtime plans, acts through sandboxed tools, validates, records the result, and updates memory. That loop is deterministic and rule-based in this release.

## Recovery As Engineering

Brains recover from disruption in many ways. Onyx Brain treats recovery as an engineering requirement: journal risky actions, snapshot before risky edits, support rollback, and run doctor checks when state may be unhealthy.

## Limits

Onyx Brain is not conscious, not AGI, not a neural network, and not a biological model. It is an experimental runtime skeleton.
