@smoke
Feature: View Switching

  Scenario: Switch the views from Upcoming over Quick Wins to Recent Changes and back again
    Given no tasks at all

    When I open the app
    Then I see the page title is Upcoming
    When I click the next view arrow
    Then I see the page title is Quick Wins
    When I click the next view arrow
    Then I see the page title is All Open
    When I click the next view arrow
    Then I see the page title is What I Finished
    When I click the next view arrow
    Then I see the page title is Recent Changes

    When I click the prev view arrow
    Then I see the page title is What I Finished
    When I click the prev view arrow
    Then I see the page title is All Open
    When I click the prev view arrow
    Then I see the page title is Quick Wins
    When I click the prev view arrow
    Then I see the page title is Upcoming
