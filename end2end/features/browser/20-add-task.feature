Feature: Add Task switches to All Open

  Tapping "Add Task" in any view switches to All Open before
  opening the input field, so the new task is always visible.

  # ── View switch on tap ────────────────────────────────────

  Scenario: Add Task from Upcoming switches to All Open
    Given no tasks at all
    When I open the app in Upcoming view
    And I enable edit mode
    And I tap Add Task
    Then I see the page title is All Open

  Scenario: Add Task from Quick Wins switches to All Open
    Given no tasks at all
    When I open the app in Quick Wins view
    And I enable edit mode
    And I tap Add Task
    Then I see the page title is All Open

  Scenario: Add Task from All Open stays in All Open
    Given no tasks at all
    When I open the app in All Open view
    And I enable edit mode
    And I tap Add Task
    Then I see the page title is All Open

  Scenario: Add Task from What I Finished switches to All Open
    Given no tasks at all
    When I open the app in What I Finished view
    And I enable edit mode
    And I tap Add Task
    Then I see the page title is All Open

  Scenario: Add Task from Recent Changes switches to All Open
    Given no tasks at all
    When I open the app in Recent Changes view
    And I enable edit mode
    And I tap Add Task
    Then I see the page title is All Open

  # ── Task visible after creation ───────────────────────────

  Scenario: Task created from Upcoming is visible and expanded in All Open
    Given I am logged in as "e2e-test"
    And no tasks at all
    When I open the app in Upcoming view
    And I enable edit mode
    And I tap Add Task
    And I type "Buy milk" and submit
    Then no Add Task error is shown
    And I see the page title is All Open
    And I see tasks in the list
    And the new task is expanded

  # ── Error handling ────────────────────────────────────────

  Scenario: Add Task without login shows error
    Given no user is logged in
    And no tasks at all
    When I open the app in All Open view
    And I enable edit mode
    And I tap Add Task
    And I type "Buy milk" and submit
    Then I see an Add Task error
