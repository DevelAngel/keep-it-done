Feature: Upcoming view with simulated time

  Verify weekday-dependent grouping in the Upcoming view.
  Uses the test-control time offset so the test result does not
  depend on the real wall-clock day.

  See: docs/concepts/e2e-time-simulation.md

  # ── This Week vs. Next Week boundary ────────────────────

  Scenario: On Wednesday, a task due in 2 days is This Week
    Given today is simulated as Wednesday
    And the following tasks
      | status | days ago | summary     | category | due |
      | open   | 1        | Friday task | Admin    | +2  |
    When I open the app in Upcoming view
    Then I see the Upcoming groups
      | group     | summary     |
      | This Week | Friday task |

  Scenario: On Saturday, a task due in 2 days is Next Week
    Given today is simulated as Saturday
    And the following tasks
      | status | days ago | summary     | category | due |
      | open   | 1        | Monday task | Admin    | +2  |
    When I open the app in Upcoming view
    Then I see the Upcoming groups
      | group     | summary     |
      | Next Week | Monday task |
