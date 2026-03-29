# Architecture Decision Records

Decisions are documented using [MADR 4.0.0](https://github.com/adr/madr) (Markdown Any Decision Records).

## Template

```markdown
---
status: {proposed | accepted | deprecated | superseded by [title](file.md)}
date: YYYY-MM-DD
---

# {Short title: problem and solution}

## Context and Problem Statement

{What is the problem? Why does it need a decision?}

## Decision Drivers

- {key requirement or constraint}

## Considered Options

- {option 1}
- {option 2}

## Decision Outcome

Chosen option: "{option}", because {justification}.

### Consequences

- Good, because {…}
- Bad, because {…}

## Pros and Cons of the Options

### {option 1}

- Good, because {…}
- Bad, because {…}

### {option 2}

- Good, because {…}
- Bad, because {…}

## More Information

{Implementation notes, references, links.}
```

`Decision Drivers`, `Pros and Cons of the Options`, and `More Information` are optional — omit them when they add no value.
