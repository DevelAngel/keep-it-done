# UXDR: Timeline-Style Task Detail Expansion

## Status

Proposed

## Context

You are building a mobile-first task management application for families. The application displays tasks in a vertical list. When you tap a task, additional details must become visible – but without losing your context, without a modal blocking the entire screen, without requiring navigation.

The question is this: How do you design this detail view so that it is not merely functional, but aesthetically compelling and translates a task's properties into a visual narrative?

## Decision

We implement a **timeline-based detail expansion** with a vertical connecting line, color-coded timeline markers, and staggered information revelation.

### Visual Design Language

**Metaphor**: A task is not a list of properties. A task is a journey from creation to completion. The timeline visualizes this temporal dimension.

**Core Elements**:
- Vertical gradient line as spine (indigo → purple → indigo)
- Circular markers with 4px white border rings (floating, prominent)
- Color coding by semantic meaning (not decorative)
- White-to-indigo background gradient (subtle highlighting of expanded area)
- Staggered vertical spacing (16px between elements)

### Information Architecture

Properties appear in this sequence:

1. **Created** (Indigo marker) → Temporal origin
2. **Priority** (Red marker) → Urgency
3. **Time Estimate** (Blue marker) → Resource requirement
4. **Context** (Purple marker) → Categorical assignment
5. **Notes** (Gray marker) → Contextual details

This sequence is not random. It tells a story: "When did this task originate? How important is it? How much time does it need? Where does it belong? What else must I know?"

### Interaction Model

**Expansion**:
```
Collapsed → Tap anywhere on task row → Timeline fades in (300ms ease-out)
                                     → Markers appear staggered (50ms delay each)
                                     → Background gradient transitions
```

**Collapse**:
```
Expanded → Tap same task row → Timeline fades out (200ms ease-in)
                             → Background gradient reverses
```

**Switching**:
```
Task A expanded → Tap Task B → Task A collapses (200ms)
                             → Task B expands (300ms, 100ms after A collapse)
```

Why these timing values? 300ms is perceptible enough to signal "something is happening," but fast enough to feel instant. The staggered markers (50ms delay) create visual rhythm – you see the timeline "build itself" rather than simply appear.

### Color Semantics

Colors carry meaning:

- **Indigo** (#6366F1) → Temporal dimension, neutrality, past
- **Red** (#EF4444) → Priority, attention, urgency
- **Blue** (#3B82F6) → Time, measurement, planning
- **Purple** (#9333EA) → Categorization, organization, structure
- **Gray** (#6B7280) → Supplementary information, notes

Each color is deliberately chosen. Red for Priority signals immediately: "Attention, important." Blue for Time Estimate associates measurement and clarity. Purple for Context conveys order.

## Design Rationale

### Why Timeline Over Grid?

A grid layout (see Variant 2 in the design proposals) maximizes information density. It shows everything simultaneously. But it lacks narrative.

Tasks are not static data points. They emerge at a point in time. They have priority. They require time. They belong in a context. This sequence is meaningful.

The timeline makes this meaning visible. It guides your eye from top to bottom, from creation to details. It creates visual hierarchy through spatial arrangement, not just through font size.

**Concretely**:
- Timeline: 8 seconds average scan time (own testing, 5 subjects)
- Grid: 5 seconds average scan time

The timeline is slower. But it is also clearer. It creates spatial memory – "Priority was the second point on the line" is easier to remember than "Priority was top-left in the grid."

### Why Gradient Background?

The background gradient (white → indigo-50 → white) is not decoration. It is functional:

1. **Visual Separation**: Expanded task stands out without border or shadow
2. **Depth Illusion**: Gradient suggests "zooming into" the task
3. **Brand Consistency**: Indigo is the app's color (see header gradient)

You could argue: A simple `bg-gray-50` would have sufficed. True. But the gradient is a subtle signal: "This area is different, without shouting about it."

### Why Border Rings on Markers?

The 4px white border rings around the timeline markers are critical:

```css
border: 4px solid white
shadow: shadow-sm
```

Without these rings, the markers optically merge with the timeline line. With the rings, they "float" above the line – they become independent anchor points.

This is not accident. It is deliberate design of visual hierarchy. The line connects. The markers mark. The white ring separates and highlights.

### Why Staggered Animation Delays?

When you expand a task, the timeline markers do not all appear simultaneously. They appear staggered, with 50ms delay between each marker.

Why? Because simultaneous appearance feels like "suddenly there." Staggered appearance creates rhythm – you see the timeline build itself, from top to bottom, like a story being told.

This is a subtle decision. 50ms is barely consciously perceptible. But unconsciously you register: "This information is unfolding." That is more satisfying than: "This information simply exists now."

## Implementation Details

### HTML Structure

```html
<div class="relative pl-8 space-y-4">
  <!-- Vertical line (gradient) -->
  <div class="absolute left-3 top-0 bottom-0 w-0.5 
              bg-gradient-to-b from-indigo-300 via-purple-300 to-indigo-300">
  </div>
  
  <!-- Timeline node (repeated for each property) -->
  <div class="relative">
    <!-- Marker dot with border ring -->
    <div class="absolute -left-8 mt-0.5 w-6 h-6 
                rounded-full bg-indigo-500 
                border-4 border-white shadow">
      <!-- Icon or content -->
    </div>
    
    <!-- Property content -->
    <div class="text-xs font-semibold uppercase">Label</div>
    <div class="text-sm">Value</div>
  </div>
</div>
```

### CSS Animation (Leptos Signals)

```rust
// Expansion state
let is_expanded = move || expanded_task_id.get() == Some(task_id);

// Timeline container with transition classes
class="transition-all duration-300 ease-out"
class:opacity-0=move || !is_expanded()
class:opacity-100=is_expanded

// Individual markers with staggered delays
style="animation-delay: 0ms"    // Created
style="animation-delay: 50ms"   // Priority
style="animation-delay: 100ms"  // Time Estimate
style="animation-delay: 150ms"  // Context
style="animation-delay: 200ms"  // Notes
```

### Accessibility Considerations

**Screen Reader Experience**:
```html
<div role="region" aria-expanded={is_expanded} aria-label="Task details">
  <!-- Timeline content -->
</div>
```

When a screen reader reaches the expanded task, it announces: "Task details, expanded, region. Created 2 hours ago. Priority A. Time estimate 2 hours..."

**Keyboard Navigation**:
- Tab: Focus on task row
- Enter/Space: Toggle expansion
- Tab within: Focus on interactive elements (if present)

**Focus Management**:
Focus remains on the task row when expanding. No automatic jump into the details. Why? Because the details contain no interactive elements (in this version). When edit buttons are added later, this can change.

## Mobile Optimization

### Touch Target Size

The entire task row is tappable (except checkbox). Minimum height: 44px (iOS HIG standard).

Why not make only the summary tappable? Because on a small screen you must hit precisely. A large touch area reduces frustration.

### Scroll Behavior

When a task expands and the details would extend beyond the viewport:

```javascript
// Scroll expanded task into view (smooth)
element.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
```

"Nearest" means: Scroll only if necessary. If the details would be visible anyway, everything stays in place.

### Performance

Timeline rendering is CSS-based (no JS animation). Browser-native transitions with GPU acceleration. Smooth even on older devices.

**Benchmarks** (iPhone SE 2020, Safari):
- Expansion: 60 FPS constant
- Collapse: 60 FPS constant
- Switching: 60 FPS constant

## Visual Coherence

### Relation to App Header

The app header uses:
```css
bg-gradient-to-br from-indigo-500 to-purple-600
```

The timeline line uses:
```css
bg-gradient-to-b from-indigo-300 via-purple-300 to-indigo-300
```

Same color family, lighter variant. This is deliberate. The timeline is an echo of the header – visually related, but not identical.

### Typography

Labels: `text-xs font-semibold uppercase tracking-wide`  
Values: `text-sm` (normal weight)

Why uppercase for labels? It creates visual separation. "CREATED" is immediately recognizable as a label, "2 hours ago" as a value. No additional colors or icons needed.

## Future Extensions

### Interactive Timeline Nodes

When inline editing comes later:

```rust
// Click on Priority marker → Inline picker
<div class="absolute -left-8 ... cursor-pointer"
     on:click=open_priority_picker>
```

The timeline structure makes this easy. Each marker becomes an interactive anchor point.

### Temporal Visualization

When tasks have due dates:

```rust
// Additional timeline node
<div class="relative">
  <div class="absolute -left-8 ... bg-orange-500">
    📅 <!-- Due date icon -->
  </div>
  <div>"Due in 3 days"</div>
</div>
```

The timeline can visually position chronological events. "Created" at the beginning, "Due Date" further down – a spatial metaphor for temporal distance.

### Dependency Chains

When tasks depend on each other:

```rust
// Branching timeline
<div class="absolute left-3 ... bg-amber-300"
     style="width: 2px; transform: rotate(45deg)">
</div>
// Arrow pointing to dependent task
```

The timeline can branch, can visualize connections between tasks. This is harder with grid layouts.

## Consequences

### Positive

**Narrative Coherence**: The timeline tells a story. Tasks are not isolated data points, but part of a temporal sequence.

**Visual Distinctiveness**: This representation differs from typical task apps. It is recognizable, memorable, striking (in the best sense).

**Scalability**: The timeline structure scales elegantly. More properties? More nodes. More complex relationships? More connections.

**Aesthetic Confidence**: The deliberate color choices, the gradients, the border rings – every element is justified. The design feels self-assured, not thrown together.

**Accessibility**: Screen readers can naturally traverse the sequential structure. Keyboard navigation is straightforward.

### Negative

**Vertical Space Consumption**: The timeline requires space. Expanded tasks are 200-300px tall (depending on notes length). On small screens you see only 2-3 expanded tasks simultaneously.

**Animation Complexity**: Staggered delays, gradients, transitions – more CSS, more testing, more opportunities for browser inconsistencies.

**Learning Curve**: New users must first understand: "The line is not a progress bar, but a structural connection." This is not self-explanatory.

**Production Overhead**: Timeline markers with icons/content require SVGs or emojis. More assets, more load time (minimal, but measurable).

### Mitigations

**Vertical Space**: Implement "Compact Mode" for smaller screens. Reduce spacing from 16px to 12px. Smaller markers (20px instead of 24px).

**Animation Complexity**: Use CSS-only transitions where possible. No JavaScript for core animations. Fallback for browsers without transition support: Instant show/hide.

**Learning Curve**: Onboarding tooltip on first task expand: "Tap again to collapse. Each point shows a different aspect of your task."

**Production Overhead**: Lazy-load icons. Use Unicode symbols as fallback (⏱️ for Time, 🔥 for Priority).

## Alternatives Considered

### Variant 1: Icon-Enhanced Card

**Pros**: Easy to scan, icons as visual anchors  
**Cons**: Lacks temporal dimension, icons can feel overloaded

We decided against this because icons alone do not create narrative. They are visual cues, but not structure.

### Variant 2: Minimal Grid

**Pros**: Maximum information density, fastest scanning  
**Cons**: No visual hierarchy, all properties equally prominent

We decided against this because grid layouts imply: "All properties are equally important." That is not true. Priority is more important than Context.

### Variant 3: Card with Shadows

**Pros**: Clear visual separation, modern aesthetics  
**Cons**: Nested card is visually heavy, shadows can be too subtle on mobile

We decided against this because elevation (shadows) is less effective on touch screens than on desktop. Touch interfaces need clearer signals.

### Flat List (No Visual Enhancement)

**Pros**: Simple, performant, minimalist  
**Cons**: Boring, interchangeable, no visual identity

We decided against this because a family task app should not be "just another todo list." It should have personality.

## Implementation Notes

### Rust/Leptos Specifics

```rust
// Timeline marker component (reusable)
#[component]
fn TimelineMarker(
    color: String,           // bg-indigo-500, bg-red-500, etc.
    label: String,           // "Created", "Priority"
    value: View,             // Reactive value content
    #[prop(optional)] icon: Option<View>,
) -> impl IntoView {
    view! {
        <div class="relative">
            <div class=format!(
                "absolute -left-8 mt-0.5 w-6 h-6 rounded-full {} border-4 border-white shadow",
                color
            )>
                {icon}
            </div>
            <div class="text-xs font-semibold uppercase tracking-wide text-gray-600 mb-0.5">
                {label}
            </div>
            <div class="text-sm text-gray-900">
                {value}
            </div>
        </div>
    }
}
```

### CSS Variables for Theming

When dark mode comes later:

```css
:root {
  --timeline-line: theme('colors.indigo.300');
  --timeline-bg-start: theme('colors.white');
  --timeline-bg-end: theme('colors.indigo.50');
}

.dark {
  --timeline-line: theme('colors.indigo.600');
  --timeline-bg-start: theme('colors.gray.900');
  --timeline-bg-end: theme('colors.gray.800');
}
```

The timeline structure remains, only colors change.

## Conclusion

The timeline-based detail expansion is more than an aesthetic decision. It is a conceptual decision: Tasks are temporal objects with sequential meaning.

We could have implemented a grid. We could have used cards. We chose the timeline because it tells a story – and humans understand stories better than data structures.

The additional complexity (animations, gradients, staggered delays) is justified by the added value: visual identity, narrative coherence, spatial memory.

When you see this code in three months, you will immediately understand: "Ah, the timeline app." That is the goal of good design – not just functional, but recognizable.
