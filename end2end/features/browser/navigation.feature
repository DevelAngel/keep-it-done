@smoke @wasm
Feature: View Switching

  Scenario: Switch the views from Upcoming over Quick Wins to Recent Changes
    Given the following tasks
      | status | days ago | summary                                      | category  | context  | estimate | priority | note                                   |
      | open   | 21       | Hang picture frames (in the box since March) | Household | @home    |          |          |                                        |
      | open   | 18       | Sort through mail pile on counter            | Household | @home    | 15m      |          |                                        |
      | open   | 16       | Cancel unused gym membership                 | Admin     | @home    | 15m      |          | Check cancellation period first        |
      | open   | 14       | Mow the lawn                                 | Household | @garden  | 45m      | B        |                                        |
      | open   | 12       | Research better phone plan                   | Admin     | @home    | 45m      |          |                                        |
      | open   | 10       | Fix squeaky hallway door                     | Household | @home    | 30m      |          |                                        |
      | open   | 9        | File tax documents                           | Admin     | @home    | 1h       | A        | Deadline is next month                 |
      | done   | 8        | Paid electricity bill                        | Admin     |          |          |          |                                        |
      | open   | 7        | Water the houseplants                        | Household | @home    | 15m      |          |                                        |
      | open   | 6        | Meal prep for the week                       | Household | @home    | 2h       |          |                                        |
      | done   | 6        | Picked up prescription                       | Errands   |          |          |          |                                        |
      | open   | 5        | Book car inspection                          | Admin     | @phone   |          | B        |                                        |
      | open   | 4        | Reply to school email about field trip       | Kids      | @home    | 15m      |          |                                        |
      | done   | 4        | Fixed leaky bathroom faucet                  | Household |          |          |          |                                        |
      | open   | 3        | Buy birthday present for Emma                | Family    | @errands | 30m      |          | She mentioned she likes art supplies   |
      | done   | 3        | Helped Leo with science project              | Kids      |          |          |          |                                        |
      | open   | 2        | Clean out the fridge                         | Household | @home    | 30m      |          |                                        |
      | open   | 1        | Take out the recycling                       | Household | @home    | 15m      |          |                                        |
      | open   | 1        | Plan something for the long weekend          | Family    | @home    |          |          |                                        |
      | done   | 1        | Brought bottles to recycling center          | Household |          |          |          |                                        |
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
