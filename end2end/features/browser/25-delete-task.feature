Feature: Delete Task via inline confirm

  A task can be deleted from the expanded detail panel using a
  two-tap inline-confirm button (Idle → Armed → deleted).

  # ── Happy path ───────────────────────────────────────────────

  Scenario: Delete a task removes it from the list
    Given I am logged in as "e2e-test"
    And the following tasks
      | summary      | category | status | days ago |
      | Buy bananas  | Inbox    | open   | 1        |
      | Fix doorbell | Inbox    | open   | 1        |
    When I open the app in All Open view
    Then I see tasks in the list
    When I expand the task "Buy bananas"
    And I tap the delete button
    And I confirm the deletion
    Then I do not see the task "Buy bananas"
    And the task details are collapsed

  # ── Filesystem cleanup ───────────────────────────────────────

  Scenario: Deleted task file is removed after flush
    Given I am logged in as "e2e-test"
    And the following tasks
      | summary      | category | status | days ago |
      | Mow the lawn | Inbox    | open   | 1        |
    When I open the app in All Open view
    Then I see tasks in the list
    When I expand the task "Mow the lawn"
    And I tap the delete button
    And I confirm the deletion
    And I flush the task cache
    Then only 0 task files remain on disk

  Scenario: Non-deleted task files survive flush
    Given I am logged in as "e2e-test"
    And the following tasks
      | summary      | category | status | days ago |
      | Task alpha   | Inbox    | open   | 1        |
      | Task beta    | Inbox    | open   | 2        |
    When I open the app in All Open view
    Then I see tasks in the list
    When I expand the task "Task alpha"
    And I tap the delete button
    And I confirm the deletion
    And I flush the task cache
    Then only 1 task files remain on disk

  # ── Auto-disarm ──────────────────────────────────────────────

  Scenario: Delete button auto-disarms after 3 seconds
    Given I am logged in as "e2e-test"
    And the following tasks
      | summary      | category | status | days ago |
      | Water plants | Inbox    | open   | 1        |
    When I open the app in All Open view
    Then I see tasks in the list
    When I expand the task "Water plants"
    And I tap the delete button
    And I wait for the disarm timeout
    Then the delete button shows idle state
    And I still see the task "Water plants"

  # ── Single tap does not delete ───────────────────────────────

  Scenario: Single tap on delete does not remove the task
    Given I am logged in as "e2e-test"
    And the following tasks
      | summary      | category | status | days ago |
      | Call plumber | Inbox    | open   | 1        |
    When I open the app in All Open view
    Then I see tasks in the list
    When I expand the task "Call plumber"
    And I tap the delete button
    Then the delete button shows armed state
    And I still see the task "Call plumber"
