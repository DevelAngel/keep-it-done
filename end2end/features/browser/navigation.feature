@smoke @wasm
Feature: View Switching

  Scenario: Switch the views from Upcoming over Quick Wins to Recent Changes
    Given no tasks at all
    When I open the app
    Then I see the page title is Upcoming
    And I save a screenshot as "task-list-upcoming"
    When I click the next view arrow
    Then I see the page title is Quick Wins
    And I save a screenshot as "task-list-quickwins"
    When I click the next view arrow
    Then I see the page title is All Open
    And I save a screenshot as "task-list-allopen"
    When I click the next view arrow
    Then I see the page title is What I Finished
    And I save a screenshot as "task-list-whatifinished"
    When I click the next view arrow
    Then I see the page title is Recent Changes
    And I save a screenshot as "task-list-recentchanges"
