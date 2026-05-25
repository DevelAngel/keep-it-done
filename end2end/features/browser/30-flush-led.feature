Feature: Flush Status LED

  The flush status LED shows a brief green indicator after a
  successful flush and a persistent red indicator on failure.
  Clicking the red LED opens an error detail panel.

  # -- Success path ------------------------------------------------

  Scenario: Green LED appears after successful flush
    Given I am logged in as "e2e-test"
    And the following tasks
      | summary     | category | status | days ago |
      | Buy bananas | Inbox    | open   | 1        |
    When I open the app in All Open view
    Then I see tasks in the list
    And the flush LED is hidden
    When I flush the task cache
    Then the flush LED shows success

  Scenario: Green LED auto-dismisses after 3 seconds
    Given I am logged in as "e2e-test"
    And the following tasks
      | summary   | category | status | days ago |
      | Fix fence | Inbox    | open   | 1        |
    When I open the app in All Open view
    Then I see tasks in the list
    When I flush the task cache
    Then the flush LED shows success
    When I wait for the LED to auto-dismiss
    Then the flush LED is hidden

  # -- Nothing to flush --------------------------------------------

  Scenario: No LED when nothing was dirty
    Given I am logged in as "e2e-test"
    And the following tasks
      | summary    | category | status | days ago |
      | Clean desk | Inbox    | open   | 1        |
    When I open the app in All Open view
    Then I see tasks in the list
    # Flush once to clear dirty state from switch_dir
    When I flush the task cache
    And I wait for the LED to auto-dismiss
    # Second flush has nothing dirty
    When I flush the task cache
    And I wait briefly for any event
    Then the flush LED is hidden

  # -- Error path --------------------------------------------------

  Scenario: Red LED appears on flush error and panel opens on click
    Given I am logged in as "e2e-test"
    And the following tasks
      | summary     | category | status | days ago |
      | Water plants| Inbox    | open   | 1        |
    When I open the app in All Open view
    Then I see tasks in the list
    When I make the tasks directory read-only
    And I flush the task cache
    Then the flush LED shows error
    When I click the flush LED
    Then the flush error panel is visible
    When I restore the tasks directory permissions
    And I flush the task cache
    Then the flush LED shows success
    And the flush error panel is not visible
