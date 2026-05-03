@screenshots
Feature: Capture README Screenshots

  Screenshots are captured at Pixel 8 viewport (412 x 915)
  and saved to the workspace screenshots/ directory.

  Scenario Outline: Capture <view> view
    When I open the view <param>
    Then I see the page title is <view>
    And I save a screenshot as <filename>

    Examples:
      | view            | param     | filename                       |
      | Upcoming        | upcoming  | task-list-upcoming.png         |
      | Quick Wins      | quickwins | task-list-quickwins.png        |
      | All Open        | allopen   | task-list-allopen.png          |
      | What I Finished | finished  | task-list-whatifinished.png    |
      | Recent Changes  | recent    | task-list-recentchanges.png    |

  Scenario: Capture detail expansion
    When I open the view allopen with expand first
    And I save a screenshot as task-detail-expansion.png
