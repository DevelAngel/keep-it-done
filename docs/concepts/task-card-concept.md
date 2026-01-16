# Task Card Data Model Concept

## Abstract

A task card contains the essential information needed to answer: "What should I work on next?" Core properties: action description, due date, time estimate, priority, context/category, dependencies, and notes.

## Purpose

Define the data model for task cards in a digital task management system. Focus on minimum viable information for effective task prioritization and execution.

## Core Principle

Users need to quickly evaluate which task to tackle next. The card must provide all decision-making information without requiring additional lookups or mental overhead.

## Essential Properties

**Action Description**

- Specific, action-oriented statement of what needs to be done
- Example: "Draft presentation outline for Project X" instead of "Project X"
- Eliminates cognitive friction between reminder and action

**Due Date**

- Hard deadline (immovable) or target date (flexible)
- Distinguishing between these types supports realistic prioritization
- Enables calendar-aware task selection

**Time Estimate**

- Rough estimate: "15 min", "1 hour", "half day"
- Enables matching tasks to available time windows
- Precision not required; categories sufficient

**Priority Level**

- Importance independent of urgency (A/B/C or 1/2/3)
- Prevents important work from being postponed for merely urgent tasks
- Simple classification avoids decision paralysis

**Context/Category**

- Project, life domain, or area of responsibility
- Enables batching similar tasks to reduce context-switching
- Supports hierarchical organization when beneficial

**Dependencies**

- What blocks this task from being started
- What other tasks depend on this one
- Keeps blocked tasks from cluttering the active list

**Notes**

- Supplementary information: references, details, progress updates
- Kept separate from core properties to avoid overwhelming quick-scan view
- Serves as task-specific scratchpad

## Design Rationale

The model supports two interaction modes:

- **Scanning mode**: Rapid evaluation of many tasks to find the right one
- **Execution mode**: Access to all relevant details for the selected task

This explains why some properties are essential for overview while others remain in supplementary notes.

## Extensibility

This model is a foundation, not a complete specification. Common extensions may include:

- Recurrence patterns for repeating tasks
- Attachments or file references
- Collaboration metadata
- Multi-step completion tracking

Any extension should serve the core purpose: helping users decide what to work on next.
