<objective>
Analyze the current AI fleeing/wandering system to identify performance bottlenecks and logic issues before making improvements.

This analysis will inform the subsequent optimization phases and ensure we fix the right problems.
</objective>

<context>
This is a Bevy 0.16.1 game with AI agents that flee from the player and wander when safe. The AI system is located in `src/ai/flee.rs`.

Read and thoroughly examine:

- `src/ai/flee.rs` - Main AI behavior implementation
- `src/ai/AGENTS.md` - Documentation of AI concepts
- `src/utils.rs` - Math utilities used by AI (line intersection)
- `src/collisions.rs` - Collision system the AI interacts with

Reference the CLAUDE.md and AGENTS.md files for project conventions.
</context>

<analysis_requirements>
Thoroughly analyze the following aspects:

### 1. Performance Analysis

- Count the number of raycasts per AI per frame (LOS check + 16 direction checks)
- Identify O(n) complexity issues (iterating all polygons/edges)
- Note any redundant calculations or allocations in hot paths
- Identify opportunities for spatial partitioning or caching
- Check for unnecessary per-frame allocations (Vec creation, sorting, etc.)

### 2. Behavior Logic Analysis

- Evaluate the flee direction calculation (is "opposite of player" optimal?)
- Assess the wander behavior smoothness and predictability
- Analyze the blend transition (does it feel natural?)
- Check edge cases: what happens when AI is cornered? When all directions blocked?
- Evaluate the 16-direction sampling (is it sufficient? too many?)

### 3. Code Quality Analysis

- Identify any Bevy anti-patterns (per AGENTS.md guidelines)
- Note opportunities to split the large system function
- Check for magic numbers that should be configurable
- Identify any potential floating-point precision issues

### 4. Steering Behavior Analysis

- Evaluate the steering scale and max speed values
- Check if acceleration/deceleration feels responsive
- Analyze how collision resolution interacts with steering
  </analysis_requirements>

<output_format>
Create a detailed analysis document with:

1. **Executive Summary**: Top 3-5 issues to address
2. **Performance Bottlenecks**: Ranked by impact
3. **Logic Issues**: Behavioral problems observed
4. **Recommendations**: Specific changes for each phase

Save the analysis to: `./docs/ai-system-analysis.md`
</output_format>

<verification>
Before completing, verify:
- [ ] All source files have been read and analyzed
- [ ] Performance complexity has been calculated (Big O notation)
- [ ] At least 3 concrete performance improvements identified
- [ ] At least 3 concrete logic improvements identified
- [ ] Recommendations are specific and actionable
</verification>

<success_criteria>

- Analysis document exists at `./docs/ai-system-analysis.md`
- Document contains specific line numbers and code references
- Recommendations are prioritized and feasible
- Analysis will enable informed decisions for phases 2-4
  </success_criteria>
