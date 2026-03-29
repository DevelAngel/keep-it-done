# Task Card Data Model Concept

## Abstract

A task card contains the essential information needed to answer: "What should I work on next?" Core properties: summary, due date, start date, time estimate, priority, context/category, and notes.

## Purpose

Define the data model for task cards in a digital task management system. Focus on minimum viable information for effective task prioritization and execution.

## Core Principle

Users need to quickly evaluate which task to tackle next. The card must provide all decision-making information without requiring additional lookups or mental overhead.

## Essential Properties

**Summary**

- Specific, action-oriented statement of what needs to be done
- Example: "Draft presentation outline for Project X" instead of "Project X"
- Eliminates cognitive friction between reminder and action

**Due Date**

- Either a precise datetime or a free-text estimate ("next Friday", "end of month")
- Precise dates enable sorting and calendar-aware task selection
- Free-text estimates (`Guess`) are accepted when the user cannot commit to an exact date

**Start Date**

- Earliest date on which the task can be started
- Either precise or free-text, same encoding as due date
- Allows tasks to be scheduled in advance without cluttering today's view

**Time Estimate**

- Either a precise duration (in seconds) or a free-text estimate ("15 min", "half day")
- Enables matching tasks to available time windows
- Precision not required; free-text estimates are explicitly supported

**Priority Level**

- Importance independent of urgency: `A`, `B`, or `C`
- Prevents important work from being postponed for merely urgent tasks
- Simple three-level classification avoids decision paralysis

**Context/Category**

- Project, life domain, or area of responsibility (free-text label)
- Enables batching similar tasks to reduce context-switching
- Supports hierarchical notation when beneficial (e.g. `Work/ProjectX`)

**Notes**

- Supplementary information: references, details, progress updates
- Kept separate from core properties to avoid overwhelming quick-scan view
- Serves as task-specific scratchpad

## Status

Every task has a status with an associated timestamp:

- `ToDo` — task is pending; `since` records when the status was set (typically creation time)
- `Done` — task is completed; `since` records when it was marked done

There is intentionally no "in-progress" state. Two states keep the model simple and avoid the maintenance overhead of tracking partially-completed work.

## Design Rationale

The model supports two interaction modes:

- **Scanning mode**: Rapid evaluation of many tasks to find the right one
- **Execution mode**: Access to all relevant details for the selected task

This explains why some properties are optional (omitted fields do not appear in the JSON) while notes remain separate to avoid overwhelming the quick-scan view.

## Out of Scope

The following were considered but are not part of the current model:

- **Dependencies** — tracking which tasks block others adds graph-maintenance complexity without clear benefit at family scale
- **Recurrence patterns** — repeating tasks are created manually for now
- **Attachments** — file references are noted in the `notes` field as plain text
- **Collaboration metadata** — no multi-user assignment or comment threads

## Extensibility

This model is a foundation, not a complete specification. Any extension should serve the core purpose: helping users decide what to work on next.
