# Task Detail Design Variants

## Abstract

This document presents four design variants for the expanded task detail view, each optimized for different aesthetic preferences and information density. All variants use Tailwind CSS and maintain mobile-first principles.

## Table of Contents

- [Variant 1: Card-Style with Icons](#variant-1-card-style-with-icons)
- [Variant 2: Minimal Grid Layout](#variant-2-minimal-grid-layout)
- [Variant 3: Card with Subtle Shadows](#variant-3-card-with-subtle-shadows)
- [Variant 4: Timeline-Style (Recommended)](#variant-4-timeline-style-recommended)
- [Comparison Matrix](#comparison-matrix)
- [Recommendation](#recommendation)
- [Implementation Priority](#implementation-priority)

---

## Variant 1: Card-Style with Icons

**Visual Character**: Clean, modern, icon-enhanced  
**Information Density**: Medium  
**Best For**: Users who prefer visual scanning over reading

### Implementation

```rust
<Show when=is_expanded>
    <div class="px-6 pb-4 pt-2 bg-gradient-to-b from-indigo-50 to-white">
        <div class="space-y-3">
            // Created timestamp with clock icon
            <div class="flex items-center gap-3 text-sm">
                <div class="flex-shrink-0 w-8 h-8 rounded-full bg-indigo-100 flex items-center justify-center">
                    <svg class="w-4 h-4 text-indigo-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"/>
                    </svg>
                </div>
                <div class="flex-1">
                    <div class="text-xs text-gray-500 font-medium uppercase tracking-wide">"Created"</div>
                    <div class="text-gray-900">{created_display}</div>
                </div>
            </div>

            // Priority with badge
            <div class="flex items-center gap-3 text-sm">
                <div class="flex-shrink-0 w-8 h-8 rounded-full bg-red-100 flex items-center justify-center">
                    <svg class="w-4 h-4 text-red-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 10l7-7m0 0l7 7m-7-7v18"/>
                    </svg>
                </div>
                <div class="flex-1">
                    <div class="text-xs text-gray-500 font-medium uppercase tracking-wide">"Priority"</div>
                    <div class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-gradient-to-r from-red-500 to-orange-500 text-white">
                        {mock_priority}
                    </div>
                </div>
            </div>

            // Time estimate with hourglass
            <div class="flex items-center gap-3 text-sm">
                <div class="flex-shrink-0 w-8 h-8 rounded-full bg-blue-100 flex items-center justify-center">
                    <svg class="w-4 h-4 text-blue-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"/>
                    </svg>
                </div>
                <div class="flex-1">
                    <div class="text-xs text-gray-500 font-medium uppercase tracking-wide">"Estimate"</div>
                    <div class="text-gray-900">{mock_estimate}</div>
                </div>
            </div>

            // Context with folder icon
            <div class="flex items-center gap-3 text-sm">
                <div class="flex-shrink-0 w-8 h-8 rounded-full bg-purple-100 flex items-center justify-center">
                    <svg class="w-4 h-4 text-purple-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"/>
                    </svg>
                </div>
                <div class="flex-1">
                    <div class="text-xs text-gray-500 font-medium uppercase tracking-wide">"Context"</div>
                    <div class="inline-flex items-center px-2.5 py-0.5 rounded-md text-xs font-medium bg-purple-100 text-purple-800">
                        {mock_context}
                    </div>
                </div>
            </div>

            // Notes section (if present)
            {mock_notes.map(|notes| view! {
                <div class="pt-3 mt-3 border-t border-indigo-200">
                    <div class="flex items-start gap-3">
                        <div class="flex-shrink-0 w-8 h-8 rounded-full bg-gray-100 flex items-center justify-center">
                            <svg class="w-4 h-4 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"/>
                            </svg>
                        </div>
                        <div class="flex-1 min-w-0">
                            <div class="text-xs text-gray-500 font-medium uppercase tracking-wide mb-1">"Notes"</div>
                            <div class="text-sm text-gray-700 leading-relaxed whitespace-pre-wrap">{notes}</div>
                        </div>
                    </div>
                </div>
            })}
        </div>
    </div>
</Show>
```

### Design Notes

- **Icons**: Each property has a semantically meaningful icon
- **Icon circles**: Colored backgrounds match property importance (red for priority, blue for time)
- **Labels**: Small caps provide visual hierarchy
- **Background gradient**: Subtle indigo gradient distinguishes expanded area
- **Spacing**: Generous gaps (gap-3) for touch-friendly interaction

---

## Variant 2: Minimal Grid Layout

**Visual Character**: Dense, information-focused, scannable  
**Information Density**: High  
**Best For**: Power users who want maximum information, minimum space

### Implementation

```rust
<Show when=is_expanded>
    <div class="px-6 pb-4 pt-3 bg-gray-50 border-t border-gray-200">
        // Two-column grid for properties
        <div class="grid grid-cols-2 gap-x-4 gap-y-3 text-xs mb-3">
            // Created
            <div>
                <dt class="text-gray-500 font-medium mb-0.5">"Created"</dt>
                <dd class="text-gray-900">{created_display}</dd>
            </div>

            // Priority
            <div>
                <dt class="text-gray-500 font-medium mb-0.5">"Priority"</dt>
                <dd>
                    <span class="inline-flex items-center justify-center w-6 h-6 rounded bg-gradient-to-br from-red-500 to-orange-500 text-white text-xs font-bold">
                        {mock_priority}
                    </span>
                </dd>
            </div>

            // Time estimate
            <div>
                <dt class="text-gray-500 font-medium mb-0.5">"Estimate"</dt>
                <dd class="text-gray-900">{mock_estimate}</dd>
            </div>

            // Context
            <div>
                <dt class="text-gray-500 font-medium mb-0.5">"Context"</dt>
                <dd>
                    <span class="inline-block px-2 py-0.5 rounded text-xs bg-indigo-100 text-indigo-800">
                        {mock_context}
                    </span>
                </dd>
            </div>
        </div>

        // Notes full-width
        {mock_notes.map(|notes| view! {
            <div class="pt-3 border-t border-gray-300">
                <dt class="text-gray-500 font-medium text-xs mb-1">"Notes"</dt>
                <dd class="text-sm text-gray-700 leading-snug whitespace-pre-wrap">{notes}</dd>
            </div>
        })}
    </div>
</Show>
```

### Design Notes

- **Grid layout**: 2-column grid maximizes space efficiency
- **Definition list semantics**: Proper `<dt>`/`<dd>` for accessibility
- **Tight spacing**: Minimal padding focuses on information
- **Subtle background**: Light gray distinguishes from main list
- **Notes span full width**: Accommodates longer text

---

## Variant 3: Card with Subtle Shadows

**Visual Character**: Elevated, modern, card-like  
**Information Density**: Low-Medium  
**Best For**: Users who prefer clear visual separation and breathing room

### Implementation

```rust
<Show when=is_expanded>
    <div class="px-4 pb-4 pt-2">
        <div class="bg-white rounded-lg shadow-sm border border-gray-200 p-4 space-y-3">
            // Each property as a mini-card
            <div class="flex items-center justify-between py-2 px-3 bg-gray-50 rounded-md">
                <span class="text-sm font-medium text-gray-700">"Created"</span>
                <span class="text-sm text-gray-900">{created_display}</span>
            </div>

            <div class="flex items-center justify-between py-2 px-3 bg-gray-50 rounded-md">
                <span class="text-sm font-medium text-gray-700">"Priority"</span>
                <div class="flex items-center gap-2">
                    <span class="inline-flex items-center justify-center w-7 h-7 rounded-full bg-gradient-to-br from-red-500 to-orange-500 text-white text-xs font-bold shadow-sm">
                        {mock_priority}
                    </span>
                </div>
            </div>

            <div class="flex items-center justify-between py-2 px-3 bg-gray-50 rounded-md">
                <span class="text-sm font-medium text-gray-700">"Time Estimate"</span>
                <span class="text-sm text-gray-900">{mock_estimate}</span>
            </div>

            <div class="flex items-center justify-between py-2 px-3 bg-gray-50 rounded-md">
                <span class="text-sm font-medium text-gray-700">"Context"</span>
                <span class="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium bg-indigo-500 text-white shadow-sm">
                    {mock_context}
                </span>
            </div>

            // Notes as expanded section
            {mock_notes.map(|notes| view! {
                <div class="pt-3 mt-3 border-t border-gray-200">
                    <div class="text-xs font-medium text-gray-700 mb-2 uppercase tracking-wide">"Notes"</div>
                    <div class="text-sm text-gray-900 leading-relaxed p-3 bg-amber-50 border border-amber-200 rounded-md whitespace-pre-wrap">
                        {notes}
                    </div>
                </div>
            })}
        </div>
    </div>
</Show>
```

### Design Notes

- **Nested card**: Inner card creates depth with shadow
- **Row-based layout**: Each property is a horizontal bar
- **Hover states**: Could add hover effects on individual rows
- **Notes highlighted**: Amber background draws attention to free text
- **Shadows**: Subtle shadows create hierarchy

---

## Variant 4: Timeline-Style (Implemented)

**Visual Character**: Narrative, timeline-like progression
**Information Density**: Medium
**Best For**: Tasks with temporal or sequential properties

### Implementation

```rust
<Show when=is_expanded>
    <div class="px-6 pb-4 pt-3 bg-gradient-to-b from-slate-900 via-slate-800 to-slate-900">
        <div class="relative pl-8 space-y-4">
            // Vertical line (cyan → teal → cyan)
            <div class="absolute left-3 top-0 bottom-0 w-0.5 bg-gradient-to-b from-cyan-500 via-teal-500 to-cyan-500"></div>

            // Priority (optional, color depends on level A/B/C)
            <div class="relative">
                <div class="absolute -left-8 mt-0.5 w-6 h-6 rounded-full bg-red-500 border-4 border-slate-900 shadow flex items-center justify-center">
                    <span class="text-white text-xs font-bold">"A"</span>
                </div>
                <div class="text-xs font-semibold text-red-400 uppercase tracking-wide mb-0.5">"Priority"</div>
                <div class="text-sm text-slate-200">"Critical"</div>
            </div>

            // Due Date, Start Date, Time Estimate, Context, Notes
            // (each optional — shown only when set)

            // Created (always present, always last)
            <div class="relative">
                <div class="absolute -left-8 mt-0.5 w-6 h-6 rounded-full bg-cyan-700 border-4 border-slate-900 shadow flex items-center justify-center">
                    <div class="w-2 h-2 rounded-full bg-white"></div>
                </div>
                <div class="text-xs font-semibold text-cyan-400 uppercase tracking-wide mb-0.5">"Created"</div>
                <div class="text-sm text-slate-200">{created_display}</div>
            </div>
        </div>
    </div>
</Show>
```

### Design Notes

- **Timeline metaphor**: Vertical progression with connecting line (cyan → teal → cyan)
- **Dot indicators**: Colored markers per semantic meaning — red/amber/sky for priority, teal for dates, amber for estimate, slate for notes, cyan for created
- **Border rings**: `border-slate-900` rings separate markers from background (matches page background)
- **Gradient background**: Slate wash distinguishes expanded area from task list
- **Created last**: Origin anchors the bottom — actionable fields first, archival last

---

## Comparison Matrix

| Feature | Variant 1 (Icons) | Variant 2 (Grid) | Variant 3 (Cards) | Variant 4 (Timeline) |
|---------|-------------------|------------------|-------------------|----------------------|
| **Visual Weight** | Medium-High | Low | High | Medium |
| **Scanning Speed** | Fast | Very Fast | Medium | Medium |
| **Mobile Friendly** | Excellent | Good | Good | Excellent |
| **Information Density** | Medium | High | Low-Medium | Medium |
| **Aesthetic** | Modern, Clean | Minimal, Dense | Elevated, Spacious | Narrative, Flow |
| **Best For** | General use | Power users | Visual preference | Story-driven tasks |
| **Complexity** | Medium | Low | Medium | High |

---

## Recommendation

**For your use case** (family task management with AI assistant), we recommend **Variant 4 (Timeline-Style)** with the following rationale:

### Why Timeline Works Best

**Narrative flow**: Tasks have a temporal dimension (created → priority → estimate → context). Timeline visualizes this naturally.

**Visual hierarchy**: The connecting line guides the eye downward. Properties reveal themselves in logical order.

**Scannable but detailed**: Fast to scan (colored dots), deep when needed (expanded text).

**Future-proof**: Easy to add future properties (due date, dependencies) as additional timeline nodes.

**Distinctive**: Unlike typical task apps. Memorable. Reflects the "conversation with AI" philosophy.

### Adaptation

The implemented version combines Timeline's structure with Variant 1's icon approach: each marker contains either a letter (Priority) or an SVG icon (Due Date, Start Date, Time Estimate, Context, Notes), or a white dot (Created). This gives timeline flow + semantic icons + minimal visual noise.

---

## Implementation Decision

**Variant 4 (Timeline-Style) was chosen and implemented.** See `docs/uxdr/timeline-detail-expansion.md` for the full rationale. Variants 1–3 remain as historical design alternatives.
