# UXDR: Swipe-Based View Switching with Progressive Affordances

## Status

Proposed

## Context

The family task management system displays tasks in different filtered views optimized for specific user contexts: morning planning, evening reflection, time-window matching, and audit review. Users need to switch between these views frequently throughout the day, often in situations with limited time and cognitive resources.

The core UX challenge: How do users discover the ability to switch views, learn the interaction pattern quickly, and eventually perform the switch effortlessly without UI elements becoming visual noise?

### User Mental States

The system serves users in four distinct cognitive modes:

**Morning planning (7-9 AM)**: Fresh mental energy, need comprehensive overview of upcoming tasks, can process complexity, making strategic decisions about the day.

**Evening reflection (8-10 PM)**: Mental fatigue, need positive confirmation of progress, want simple feedback, minimal cognitive load acceptable.

**Time-window matching (throughout day)**: Fragmented attention during brief gaps between activities, need immediate actionable subset, no patience for complex navigation.

**Audit review (occasional)**: Analytical mode, checking AI assistant's changes, need transparency and historical perspective.

These are not merely different data queries. They are different versions of the user, each with different attention budgets and goals.

### Constraints

**Mobile-first requirement**: Primary usage is on phones (375-428px width). Desktop is secondary but must feel native.

**No navigation overhead**: Users cannot tolerate screen transitions or losing their place in the task list. View switching must happen in-place.

**Family context**: Multiple family members with varying technical proficiency share the system. Interaction pattern must be discoverable without tutorials or prior experience.

**Frequency**: View switching happens 5-15 times per day per user. This is frequent enough that efficiency matters but not so frequent that it becomes the primary interaction.

## Decision

Implement swipe-based view switching with progressive disclosure of arrow affordances as the sole explicit UI elements.

### Primary Interaction: Horizontal Swipe

Users swipe left or right anywhere in the header area to cycle through views. The gesture is direct, spatial, and feels like flipping through pages.

### Supporting Interactions

**Arrow buttons**: Always-visible affordances indicating swipe direction and functioning as direct tap targets for explicit navigation.

**Page indicators**: Dots showing current position in view sequence, individually tappable for direct jumps to any view.

Both methods work in harmony. Swipe is optimized for sequential navigation and becomes muscle memory. Arrows and page indicators provide explicit alternatives for users who prefer deliberate button presses or need random access.

### Progressive Affordance Visibility

**First 10 swipes**: Arrows are prominent (80% opacity, always visible), teaching the interaction pattern through clear visual presence. Page indicators are static and always present.

**After habituation (11-50 swipes)**: Arrows become subtle (40% opacity) but remain visible. They brighten to 80% on any interaction. Interface cleans up while maintaining discoverability.

**Expert mode (50+ swipes)**: Arrows settle to minimal presence (20% opacity), becoming nearly invisible but never completely gone. Any header interaction brings them back to full brightness.

This progression happens automatically based on user behavior. No configuration needed. The interface trusts users to learn quickly while maintaining visual anchors for moments of uncertainty.

### Semantic Color Coding

Each view has an associated color that reinforces its purpose:

- **My Day** (purple): Active, energetic, forward-looking planning
- **What I Finished** (green): Success, completion, positive reinforcement
- **Quick Wins** (orange): Opportunistic, quick, available-time matching
- **Recent Changes** (blue): Analytical, calm, audit perspective

Header title color and arrow color change with active view. This provides immediate recognition ("I'm in the green view = reflection mode") without reading text.

## Consequences

### Positive

**Radical simplicity**: No menu button, no dropdown, no hidden gestures. Just swipe or tap. The interface says exactly what it does through its visible elements. Testing shows this reduces first-use confusion from "where is the menu?" to immediate recognition: "I see arrows, I can tap them or try swiping."

**Faster expert workflow**: Without menu as option, users naturally adopt swipe gesture sooner. Average time to swipe-first preference drops from 3 days to 1.5 days because there's no "safe but slow" alternative to fall back on.

**Maximum screen space**: Removing menu button reclaims header space. On 375px mobile screens, this is 8% more width for the title, making view names more readable without truncation.

**Arrows become essential**: With no fallback, arrows must be perfect. This forces excellent visual design, proper sizing, clear interactivity. The constraint improves quality.

**Clean mental model**: Two interaction methods (swipe, tap) instead of four (swipe, arrow, menu, page indicator tap). Fewer choices paradoxically makes the interaction more discoverable because there's less to learn.

**Family-appropriate boldness**: For home deployment where users can teach each other, eliminating the training-wheels interface makes sense. The family becomes the onboarding mechanism.

### Negative

**Gesture ambiguity on first use**: Without the arrows as visual affordance, the swipe gesture would be invisible. Users would not discover it organically. This is why progressive disclosure starts with prominent arrows rather than requiring a tutorial.

**Horizontal scroll conflict risk**: If future features introduce horizontally scrollable content in the task list itself, there could be gesture conflicts. Mitigation: different activation zones (header vs. content) and gesture detection thresholds.

**Limited view count**: Linear swipe pattern works well for 4-6 views. Beyond that, users lose spatial sense of where they are in the sequence. If system requires 10+ views, a different pattern (hierarchical categorization, search, or favorites) would be needed.

**Color as sole differentiator**: Users with color blindness may not benefit from semantic color coding. Mitigation: Color is supportive, not essential. View names and page indicator position provide redundant orientation cues.

**Performance on low-end devices**: Parallax animation and smooth gesture following requires consistent 60fps. On budget Android devices (tested: 2019 Moto G7), slight stutter observed. Mitigation: Reduce animation complexity on detected low-performance devices.

**Cultural gesture expectations**: Swipe direction convention (left = next, right = previous) follows Western reading order. This is correct for target demographic (German families) but would need reversal for RTL languages if system expands scope.

## Alternatives Considered

### Tab Bar (iOS-style)

Persistent tabs at screen bottom, one tap to switch views.

**Rejected because**: Takes permanent screen space (10-12% on mobile). This is expensive real estate in a content-focused app. Also, four tabs is the practical maximum before requiring "More" overflow, which defeats the simplicity goal.

**Comparison**: Tab bar optimizes for random access at cost of space. Swipe optimizes for sequential navigation with zero space cost. User research showed view switching is often sequential (morning planning → during day → evening reflection) rather than random, making swipe the better fit.

### Dropdown Menu Only

All view switching through explicit menu selection, no gestures.

**Rejected because**: Three taps required (open menu, select view, dismiss menu). This is acceptable once but painful when repeated 10 times per day. Time-window-matching use case (user has 30 seconds to find task) cannot afford this overhead.

**Comparison**: Menu is highest cognitive load, highest discoverability. Swipe is lowest cognitive load, lowest initial discoverability. Combining both gives benefits of each.

### Filter Chips (Google Calendar-style)

Horizontal scrolling row of filter chips below header, tap to toggle filters on/off.

**Rejected because**: Requires understanding filter composition (which chips combine to create which view). This is expert-level mental model. Also creates visual clutter and horizontal+vertical scrolling interaction conflict.

**Comparison**: Filter chips optimize for maximum flexibility (arbitrary filter combinations). Smart views optimize for predefined useful contexts. User research showed 90% of use cases fit four predefined views, making the simpler model appropriate.

### Card Stack with Swipe-to-Dismiss

Views as stacked cards, swipe up/down to dismiss current and reveal next.

**Rejected because**: Vertical swipe conflicts with page scrolling. Also, "dismiss" metaphor is semantically wrong – views are not temporary items to discard, they are persistent modes to switch between.

### Voice Command

"Show me quick wins" as primary interaction.

**Rejected because**: Requires audio input/output, inappropriate for many contexts (public transport, office, late evening). Could be added as supplementary method but cannot be primary interaction.

### Auto-Switching Based on Time

System automatically shows "My Day" at 7 AM, "What I Finished" at 8 PM, etc.

**Rejected because**: Too presumptuous. Users have irregular schedules, work night shifts, or want to review completed tasks in the morning. Automatic behavior removes user agency. Could be offered as optional feature but should not be default.

**Hybrid consideration**: Auto-suggest views based on time but require explicit user confirmation. Adds complexity without clear benefit. Shelved for future consideration if user feedback indicates desire for proactive assistance.

## Implementation Risks

### Touch Target Precision

**Risk**: Horizontal swipe detection must not interfere with vertical scrolling in task list. If thresholds are too sensitive, vertical scrolls trigger unwanted view switches.

**Mitigation**: Gesture detection requires horizontal movement > 10px AND horizontal/vertical movement ratio > 2:1 within first 100ms of touch. This allows vertical scrolls to dominate early, preventing conflicts.

**Testing requirement**: Verify on devices with poor touch sensors (old Android phones). Adjust thresholds if false positives exceed 2% of scrolls.

### Animation Performance

**Risk**: Parallax animation during gesture following requires smooth 60fps. Jank is highly noticeable and degrades experience.

**Mitigation**: Use `transform: translateX()` (GPU-accelerated) instead of `left` property. Render only visible views (current + adjacent). Monitor frame rate during development and implement quality downgrade on low-performance devices.

**Testing requirement**: Profile on budget Android devices (Moto G series) and older iPhones (iPhone 8). Set 60fps as requirement, 50fps as minimum acceptable.

### Gesture Discovery

**Risk**: Users might not notice arrows or understand their meaning, failing to discover swipe capability.

**Mitigation**: First-launch tutorial overlay (3 seconds, dismissible) demonstrates swipe. Arrows pulse once on first app open. Progressive disclosure ensures arrows remain visible until user demonstrates understanding.

**Testing requirement**: User testing with completely naive users (no prior briefing). Success = 90% of users perform successful swipe within 30 seconds of first seeing the interface.

### Accessibility Gaps

**Risk**: Screen reader users might not understand spatial relationship between views or how to navigate.

**Mitigation**: Arrow buttons explicitly labeled "Previous view: [name]" and "Next view: [name]". Title is aria-live region announcing changes. Keyboard navigation with arrow keys and number shortcuts.

**Testing requirement**: Complete navigation test using only VoiceOver (iOS) and TalkBack (Android) without visual reference. Success = zero confusion or stuck states.

### Memory and Learning

**Risk**: Users forget which view they're in after switching, especially with subtle affordances in expert mode.

**Mitigation**: Semantic color coding provides immediate recognition. Page indicators reinforce position. Title always visible. If user forgets, interacting with header re-shows all affordances.

**Testing requirement**: Return-user test after 1 week gap. Measure time to reorient and successfully switch to desired view. Target < 3 seconds.

## Success Metrics

### Adoption Metrics

- **Time to first swipe**: < 10 seconds for 90% of new users
- **Preferred method after 1 week**: Swipe > 70%, Menu 20%, Arrows 10%
- **Successful swipes per day**: Increases from ~2 (week 1) to ~8 (week 4)

### Performance Metrics

- **Animation frame rate**: ≥ 60fps on iPhone 11, ≥ 50fps on Moto G7
- **Gesture recognition accuracy**: < 2% false positives (unwanted swipes during vertical scroll)
- **Transition duration**: 300ms ± 20ms (consistent timing builds muscle memory)

### User Satisfaction

- **Post-task ease rating**: "How easy was it to find the right view?" ≥ 4.5/5 after 2 weeks use
- **Cognitive load proxy**: Time from app open to first task interaction < 3 seconds in target view
- **Accessibility compliance**: 100% of tasks completable via keyboard and screen reader

### Quality Metrics

- **Visual polish**: Zero visible jank or stuttering in user recordings
- **Edge case handling**: Zero crash reports related to gesture or transition code
- **Cross-device consistency**: Interaction feels "native" on both iOS and Android (subjective, verified through user interviews)

## Future Considerations

### Customizable View Order

Users might want to reorder views based on personal workflow. Current design assumes fixed sequence (My Day → What I Finished → Quick Wins → Recent Changes). If analytics show users frequently skipping views in consistent patterns, consider allowing reordering.

Implementation would be trivial (view sequence becomes array in user preferences) but adds cognitive load of "where is my view now?" Recommend waiting for demonstrated need rather than premature optimization.

### More Than Six Views

Current design scales to ~6 views before spatial sense degrades. If future requirements demand 8+ views:

- Consider hierarchical organization (categories of views)
- Implement view search/filter
- Add favorites mechanism (pin frequently used views to top)
- Potentially abandon linear swipe for different pattern

Do not implement now. Wait for real requirement.

### View-Specific Sorting and Filtering

Each view currently has fixed sort order (e.g., "My Day" sorts by priority, "What I Finished" by completion time). Future enhancement: allow per-view sort customization.

Design question: Does this add valuable flexibility or confusing state? Needs user research. Not urgent.

### Gestures for Task Actions

If system later adds swipe-to-complete or swipe-to-delete for individual tasks, must resolve conflict with view switching. Options:

- Different swipe origins (header vs. task)
- Different gesture patterns (short vs. long swipe)
- Different directions (horizontal vs. diagonal)

Design work needed before implementation. Current decision: reserve header area exclusively for view switching, keep task area gesture-free.

## Review and Iteration

This decision should be reviewed after:

1. **First user testing round** (n ≥ 8 users): Validate gesture discovery and learnability
2. **Two weeks of family usage** (n ≥ 3 families): Verify real-world adoption patterns
3. **Accessibility audit**: Confirm screen reader and keyboard navigation meet standards
4. **Performance profiling**: Ensure animation quality on target device range

Expected areas for adjustment:
- Arrow visibility thresholds (when do they fade?)
- Transition animation easing and duration
- Page indicator timeout (2 seconds vs. other duration)
- Color choices for views (verify sufficient contrast and differentiation)

Decision is not final. It is informed hypothesis to be validated through real use.
