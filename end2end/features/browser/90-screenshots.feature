@screenshot
Feature: Screenshots of all views

  Background:
    Given today is simulated as Wednesday
    And the following tasks
      | status | days ago | summary                                      | category  | context  | estimate | priority | start | due | note                                   |
      | open   | 21       | Hang picture frames (in the box since March) | Household | #home    |          |          |       |     |                                        |
      | open   | 18       | Sort through mail pile on counter            | Household | #home    | 15m      |          | -18   |     |                                        |
      | open   | 16       | Cancel unused gym membership                 | Admin     | #home    | 15m      |          | -16   | +5  | Check cancellation period first        |
      | open   | 14       | Mow the lawn                                 | Household | #garden  | 45m      | B        | -3    |     |                                        |
      | open   | 12       | Research better phone plan                   | Admin     | #home    | 45m      |          |       |     |                                        |
      | open   | 10       | Fix squeaky hallway door                     | Household | #home    | 30m      |          | -10   |     |                                        |
      | open   | 9        | File tax documents                           | Admin     | #home    | 1h       | A        | -9    | +21 | Deadline is next month                 |
      | done   | 8        | Paid electricity bill                        | Admin     |          |          |          |       |     |                                        |
      | open   | 7        | Water the houseplants                        | Household | #home    | 15m      |          | -1    |     |                                        |
      | open   | 6        | Meal prep for the week                       | Household | #home    | 2h       |          | -2    | 0   |                                        |
      | done   | 6        | Picked up prescription                       | Errands   |          |          |          |       |     |                                        |
      | open   | 5        | Book car inspection                          | Admin     | #phone   |          | B        | -5    | +8  |                                        |
      | open   | 4        | Reply to school email about field trip       | Kids      | #home    | 15m      |          | -4    | +1  |                                        |
      | done   | 4        | Fixed leaky bathroom faucet                  | Household |          |          |          |       |     |                                        |
      | open   | 3        | Buy birthday present for Emma                | Family    | #errands | 30m      |          | -3    | +4  | She mentioned she likes art supplies   |
      | done   | 3        | Helped Leo with science project              | Kids      |          |          |          |       |     |                                        |
      | open   | 2        | Clean out the fridge                         | Household | #home    | 30m      |          |       |     |                                        |
      | open   | 1        | Take out the recycling                       | Household | #home    | 15m      |          | -1    | 0   |                                        |
      | open   | 1        | Plan something for the long weekend          | Family    | #home    |          |          | -1    | +3  |                                        |
      | open   | 10       | Renew car registration                       | Admin     | #phone   |          |          |       | -1  | Expires end of month                   |
      | done   | 1        | Brought bottles to recycling center          | Household |          |          |          |       |     |                                        |

  Scenario Outline: Make screenshot from <view> view
    When I open the app in <view> view
    Then I see the page title is <view>
    And I see tasks in the list
    And I save a screenshot for the <view> view

  Examples:
    | view            |
    | Upcoming        |
    | Quick Wins      |
    | All Open        |
    | What I Finished |
    | Recent Changes  |

  Scenario: Make screenshot of expanded task detail
    When I open the app in All Open view with expanded details
    Then I see the page title is All Open
    And I see tasks in the list
    And I save a screenshot as "task-detail-expansion"
