# Task View Switching Design

## Abstract

This document defines the interaction design for switching between filtered task views in a mobile-first family task management interface. The design uses horizontal swipe gestures combined with visual affordances (arrows and page indicators) to enable rapid context switching without navigation overhead or focus loss.

## Table of Contents

1. [Problem Space](#problem-space)
2. [Core Design Principles](#core-design-principles)
3. [Visual Design](#visual-design)
4. [Interaction Patterns](#interaction-patterns)
5. [Animation and Feedback](#animation-and-feedback)
6. [Responsive Behavior](#responsive-behavior)
7. [Accessibility Considerations](#accessibility-considerations)
8. [Implementation Notes](#implementation-notes)

## Problem Space

### User Needs

Family members interact with tasks in distinct mental states throughout the day:

- **Morning planning** (7-9 AM): Need overview of today and tomorrow, fresh mental energy, can process complexity
- **Evening reflection** (8-10 PM): Need confirmation of completed work, tired, want simple positive feedback
- **Time-window matching** (throughout day): Have 30 minutes free, need tasks matching available time
- **Audit review** (occasional): Want to see what AI assistant changed in last 7 days

These are not simply different filters. They represent different cognitive modes requiring different data subsets with minimal friction to switch between them.

### Current Limitation

Single unified task list forces users to mentally filter or manually search. This creates cognitive overhead precisely when mental energy is constrained (evening, rushed moments between commitments).

### Design Goals

1. **Zero navigation overhead**: View switching happens in-place, no screen transitions
2. **Immediate discoverability**: New users understand interaction within seconds
3. **Expert shortcuts**: Frequent users can switch views in single gesture
4. **Mobile-first**: Optimized for thumb interaction, works on smallest screens
5. **Contextual clarity**: Always clear which view is active and what it shows

## Core Design Principles

### Progressive Disclosure of Interaction

The interface reveals its interactive capabilities gradually:

**First session**: Visual affordances (arrows) are prominent, teaching interaction pattern through visibility

**After first swipe**: Affordances become subtle, interface cleans up while remaining discoverable

**After habituation**: Affordances appear on-demand (touch feedback), providing confirmation without visual noise

### Multimodal Interaction

Two ways to achieve the same goal, optimized for different contexts:

1. **Swipe gesture**: Fastest, most fluid, becomes muscle memory
2. **Direct tap**: Arrows and page indicators provide explicit buttons

Both mechanisms coexist without conflict. Swipe is primary for sequential navigation. Tap is primary for random access (page indicators) or when user prefers explicit confirmation (arrows).

### Semantic Color Coding

Each view has associated color that conveys emotional meaning:

- **My Day** → Purple: Active, energetic, forward-looking
- **What I Finished** → Green: Success, completion, positive reinforcement
- **Quick Wins** → Orange: Opportunistic, quick, energizing
- **Recent Changes** → Blue: Analytical, calm, reflective

Color is not arbitrary decoration. It reinforces the mental state appropriate for each view.

## Visual Design

### Header Anatomy

```
┌─────────────────────────────────────┐
│  ←       My Day          →          │
│          ● ○ ○ ○                    │
└─────────────────────────────────────┘
 └─┬─┘     └──┬──┘      └─┬─┘
   │          │           │
   │          │           └──────── Right arrow
   │          └──────────────────── Title (view name)
   └─────────────────────────────── Left arrow
              └──────────────────── Page indicators
```

### Arrow Specifications

**Size:**
- Mobile: 24px (touch target 48×48px minimum)
- Desktop: 18px (hover target 36×36px minimum)

**Color:**
- Inherits view color (purple for My Day, green for What I Finished, etc.)
- Base opacity: 80%
- Active state: 100%

**Position:**
- Left arrow: 20px from left edge
- Right arrow: 20px from right edge
- Vertically centered with title text

**States:**

*Default (new user):*
- Opacity: 80%
- Always visible
- Clearly recognizable as interactive buttons

*After habituation (10+ swipes):*
- Opacity: 40%
- Static
- Fade in to 80% on touch/hover

*Expert mode (50+ swipes):*
- Opacity: 20%
- Minimal but always present
- Full brightness on any interaction

*End of sequence:*
- Arrow grayed out (50% opacity of base color)
- Pre-signals no further views available
- Still responds to interaction with rubber-band

### Page Indicators

Essential navigation element showing current position in view sequence and enabling direct jumps.

**Specifications:**
- Size: 8px diameter
- Spacing: 10px between centers
- Active dot: Filled with view color
- Inactive dots: 40% opacity outline
- Touch target: 36×36px per dot

**Position:**
- Always visible below title
- 12px vertical spacing from title
- Horizontally centered

**Behavior:**
- Static, always present
- Provide constant orientation
- Direct tap for random access to any view

**Interaction:**
- Tappable on mobile (generous touch targets)
- Clickable on desktop
- Direct jump to corresponding view
- Crossfade animation if jumping more than one view

### Title Presentation

**Typography:**
- Font size: 24px (mobile), 28px (desktop)
- Font weight: 600 (semibold)
- Letter spacing: -0.02em (slight tightening for display text)

**Color:**
- Changes with active view
- Smooth transition over 300ms
- Maintains WCAG AA contrast ratio against background

**Touch/Hover State:**
- Scale: 105% (mobile touch), 102% (desktop hover)
- Transition: 100ms ease-out
- Cursor: grab (desktop)
- Arrows fade in to full opacity

## Interaction Patterns

### Swipe Gesture

**Activation:**
1. Touch down anywhere in header area (except menu icon)
2. Horizontal movement > 10px within 100ms = swipe detected
3. Vertical movement > horizontal = scroll, cancel swipe

**Following:**
- Current view follows finger with 1:1 relationship
- Next/previous view preview slides in from edge
- Rubber-band resistance at sequence boundaries (0.3 coefficient)

**Completion:**
- Release with velocity > 0.3 px/ms = commit to next view
- Release with distance > 30% of screen width = commit
- Otherwise snap back to current view
- Threshold calculated in real-time based on gesture velocity and distance

**Edge Cases:**
- At first view, swipe right: rubber-band, snap back
- At last view, swipe left: rubber-band, snap back
- Rapid successive swipes: queue transitions, maintain order

### Arrow Button Tap

**Activation:**
- Direct tap/click on arrow button
- Standard button press behavior

**Behavior:**
- Instant transition to adjacent view (no menu, no intermediary)
- Same slide animation as swipe gesture
- Haptic feedback on mobile (light impact)

**Visual Feedback:**
- Arrow scales to 110% on touch down
- Returns to 100% on touch up
- Pressed state: 50ms before transition begins
- Provides tactile confirmation

### Page Indicator Tap

**Activation:**
- Direct tap on specific dot

**Behavior:**
- Immediate transition to corresponding view
- No slide animation if jumping > 1 view
- Crossfade instead (300ms)
- Haptic feedback on touch (if available)

**Visual Feedback:**
- Tapped dot grows to 8px for 100ms before transition
- Other dots remain unchanged

## Animation and Feedback

### View Transition Animation

**Slide with Subtle Parallax:**

Outgoing view moves at 1.2× speed relative to incoming view. This creates depth perception without disorienting motion.

```
Frame 0:   Current view at x=0
Frame 1:   Current view at x=-12, next view at x=+88
Frame 2:   Current view at x=-24, next view at x=+76
...
Frame 10:  Current view at x=-120, next view at x=0
```

Duration: 300ms
Easing: cubic-bezier(0.4, 0.0, 0.2, 1) - Material Design standard deceleration

**Color Transition:**

Header background color morphs from outgoing view color to incoming view color over the same 300ms, maintaining visual continuity.

### Touch Feedback

**On Touch Down:**
- Title scales to 105% (mobile) or 102% (desktop)
- Arrows fade to 100% opacity
- Duration: 100ms, ease-out

**During Swipe:**
- Views follow finger with spring physics (stiffness: 300, damping: 30)
- Rubber-band at edges with exponential resistance
- No haptic feedback during motion (battery consideration)

**On Touch Up:**
- Scale returns to 100%
- Arrow opacity returns to contextual default
- Duration: 150ms, ease-in

**On Transition Complete:**
- Single subtle haptic tap (light impact, iOS) or vibration (50ms, Android)
- Arrows fade to contextual default after 300ms delay

**On Arrow Button Tap:**
- Arrow scales to 110% on touch down
- Returns to 100% on touch up
- Haptic feedback on touch up (confirms action)
- Immediate view transition begins

### Error States

**Boundary Feedback:**

When attempting to swipe beyond first/last view:
- View moves only 20% of finger distance
- Resistance increases exponentially
- On release: spring-back animation (200ms, elastic easing)
- Arrow at boundary pulses red briefly (150ms)

## Responsive Behavior

### Mobile (< 768px)

- All interactions optimized for thumb reach
- Touch targets minimum 48×48px (arrows), 36×36px (page indicators)
- Arrows prominent and clearly tappable
- Swipe is primary interaction for sequential navigation
- Page indicators provide random access
- Simple, clean header maximizes task list space

### Tablet (768px - 1024px)

- Similar to mobile but with more generous spacing
- Both arrows and menu icon visible simultaneously
- Hover states begin to work with Apple Pencil/stylus
- Touch targets can be slightly smaller (40×40px)

### Desktop (> 1024px)

- Arrows smaller (16px) as mouse precision is higher
- Hover states fully functional
- Keyboard navigation:
  - Arrow keys: switch views
  - Numbers 1-4: jump to specific view
  - Escape: close menu if open
- Mouse wheel over header: switch views (optional, can be disabled)
- Trackpad swipe: native gesture support

## Accessibility Considerations

### Screen Reader Support

**ARIA Labels:**

```html
<header role="navigation" aria-label="Task view selector">
  <button aria-label="Previous view: Recent Changes" class="arrow-left">
    ←
  </button>
  <h1 aria-live="polite" aria-atomic="true">
    My Day
  </h1>
  <button aria-label="Next view: What I Finished" class="arrow-right">
    →
  </button>
</header>

<nav aria-label="View indicators" class="page-indicators">
  <button aria-label="My Day" aria-current="true">●</button>
  <button aria-label="What I Finished">○</button>
  <button aria-label="Quick Wins">○</button>
  <button aria-label="Recent Changes">○</button>
</nav>
```

**Live Region:**

Title is aria-live region. When view changes, screen reader announces: "Now viewing: What I Finished. 3 tasks."

### Keyboard Navigation

**Tab Order:**
1. Left arrow button
2. Page indicator 1 (My Day)
3. Page indicator 2 (What I Finished)
4. Page indicator 3 (Quick Wins)
5. Page indicator 4 (Recent Changes)
6. Right arrow button

**Keyboard Shortcuts:**
- Arrow Left/Right: Navigate to adjacent views
- Numbers 1-4: Jump directly to view by number
- Home: Jump to first view (My Day)
- End: Jump to last view (Recent Changes)

### Reduced Motion

If user has `prefers-reduced-motion` enabled:
- No parallax effect (simple crossfade instead)
- Transition duration reduced to 150ms
- No spring physics on swipe follow
- No rubber-band animation at boundaries

### High Contrast Mode

- Arrow opacity increases to 100% always
- Page indicator active dot has 2px border
- Color transitions disabled (instant switch)
- Focus indicators: 2px solid outline

### Touch Target Sizes

All interactive elements meet WCAG 2.1 Level AAA:
- Minimum 44×44px touch targets on mobile
- Spacing between targets minimum 8px
- Entire header area (except menu icon) is swipeable

## Implementation Notes

### State Management

View switching state includes:
- `currentViewIndex`: 0-3
- `isTransitioning`: boolean
- `transitionDirection`: 'left' | 'right' | 'none'
- `touchStartX`: number | null
- `longPressTimer`: number | null

### Performance Optimization

**Virtual Views:**
Only render current view + adjacent views. Views more than 1 position away are not in DOM.

```
User at View 2:
DOM: [View 1, View 2, View 3]
Not in DOM: [View 0]
```

When transitioning to View 3:
```
Remove View 1 from DOM
Add View 4 to DOM
DOM: [View 2, View 3, View 4]
```

**GPU Acceleration:**
Use `transform: translateX()` instead of `left` property for animations. This ensures 60fps on most devices.

**Debouncing:**
Swipe gesture debounced at 16ms (60fps). Faster updates discarded to prevent jank.

### Browser Compatibility

**CSS Features:**
- CSS custom properties for color transitions
- CSS transforms for animations
- Fallback to instant transition if not supported

**JavaScript Features:**
- Touch events: TouchEvent API
- Pointer events: PointerEvent API (preferred)
- Fallback to mouse events on desktop

**Testing Matrix:**
- iOS Safari 14+
- Chrome Android 90+
- Firefox Android 90+
- Desktop Chrome 90+
- Desktop Firefox 90+
- Desktop Safari 14+

### Edge Cases

**Rapid Consecutive Swipes:**
Queue transitions. If user swipes three times quickly, all three transitions execute in sequence without conflict.

**Swipe During Transition:**
Ignore. Current transition must complete before accepting new gesture.

**Orientation Change:**
Recalculate all touch targets and thresholds. Re-render current view without transition.

**Background/Foreground:**
On app return to foreground, verify current view still valid (data may have changed). Reload if necessary without transition.

### Testing Scenarios

**New User Flow:**
1. Launch app
2. See prominent arrows
3. Tap arrow or swipe
4. Observe transition and new view
5. Notice arrows become subtle
6. Verify page indicators appear briefly

**Expert User Flow:**
1. Quick swipe gesture
2. View changes immediately
3. No visual noise from affordances
4. Muscle memory develops after ~10 uses

**Accessibility Flow:**
1. Navigate with keyboard only
2. Use screen reader
3. Verify all views reachable
4. Verify announcements clear and timely

**Error Handling:**
1. Attempt swipe at boundary
2. Observe rubber-band resistance
3. Verify no crash or stuck state
4. Arrow indicates boundary visually

### Future Considerations

**Customizable View Order:**
Users might want to reorder views based on personal workflow. Design accommodates this by making view sequence data-driven, not hardcoded.

**Additional Views:**
If more than 4 views needed:
- Consider categorization (group related views)
- Horizontal scrolling page indicators
- Or switch to dropdown-only (no swipe if >6 views)

**Personalized Views:**
Future feature where "My Day" differs per family member. Design supports this through view data being user-specific rather than global.

**Gesture Conflicts:**
If app adds other horizontal gestures (e.g., swipe to complete task), ensure different activation zones or conflict resolution logic.
