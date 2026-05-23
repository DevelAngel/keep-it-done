Feature: Upcoming view grouping

  Tasks appear in the Upcoming view grouped by temporal urgency.
  Grouping uses due_date, time_estimate, availability, and
  start_date to determine when a task needs attention.

  Weekday-dependent edge cases (This Week vs. Next Week boundaries)
  are covered by integration tests where today is controlled.
  These scenarios use only weekday-independent offsets:
  -1 (Overdue), 0 (Today), +30 (Later).

  # ── Baseline: due_date grouping ───────────────────────────

  Scenario: Overdue and today tasks appear in correct groups
    Given the following tasks
      | status | days ago | summary      | category | due |
      | open   | 3        | Overdue task | Admin    | -1  |
      | open   | 2        | Due today    | Admin    | 0   |

    When I open the app in Upcoming view
    Then I see the Upcoming groups
      | group   | summary      |
      | Overdue | Overdue task |
      | Today   | Due today    |

  # ── Urgency checkbox ──────────────────────────────────────

  Scenario: Multi-day estimate shows urgent checkbox
    Given the following tasks
      | status | days ago | summary      | category | estimate | due |
      | open   | 1        | Two-day task | Admin    | 2d       | +7  |

    When I open the app in Upcoming view
    Then the task "Two-day task" has an urgent checkbox

  Scenario: Soft-shifted task shows normal checkbox
    Given the following tasks
      | status | days ago | summary   | category | estimate | due | start |
      | open   | 1        | Soft task | Admin    | 30m      | +30 | -3    |

    When I open the app in Upcoming view
    Then the task "Soft task" has a normal checkbox

  # ── Start date: soft ranking ──────────────────────────────

  Scenario: Hard task appears before soft task in same group
    Given the following tasks
      | status | days ago | summary   | category | due | start |
      | open   | 2        | Hard task | Admin    | 0   |       |
      | open   | 1        | Soft task | Admin    | +30 | -3    |

    When I open the app in Upcoming view
    Then the task "Hard task" appears before "Soft task"

  # ── Backlog ───────────────────────────────────────────────

  Scenario: Task without dates appears in backlog beside another task
    Given the following tasks
      | status | days ago | summary      | category | due |
      | open   | 2        | Hard task    | Admin    | 0   |
      | open   | 1        | Backlog task | Admin    |     |

    When I open the app in Upcoming view
    And I expand the backlog
    Then the task "Backlog task" is in the backlog

  Scenario: Task without dates appears in backlog
    Given the following tasks
      | status | days ago | summary      | category |
      | open   | 1        | Backlog task | Admin    |

    When I open the app in Upcoming view
    And I expand the backlog
    Then the task "Backlog task" is in the backlog

  # ── Done excluded ─────────────────────────────────────────

  Scenario: Completed task does not appear
    Given the following tasks
      | status | days ago | summary       | category | due |
      | done   | 1        | Finished task | Admin    | 0   |

    When I open the app in Upcoming view
    Then I do not see the task "Finished task"

  # ── Ready to Start ────────────────────────────────────────

  Scenario: Task with only start date in Ready to Start group
    Given the following tasks
      | status | days ago | summary      | category | start |
      | open   | 1        | Started task | Admin    | 0     |

    When I open the app in Upcoming view
    Then I see the Upcoming groups
      | group          | summary      |
      | Ready to Start | Started task |
