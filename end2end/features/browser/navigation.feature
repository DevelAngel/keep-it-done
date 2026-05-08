@smoke @wasm
Feature: View Switching

  Scenario: Switch the views from Upcoming over Quick Wins to Recent Changes
    Given task "Hang picture frames (in the box since March)" in "Household" at "@home" created 21 days ago
    And task "Sort through mail pile on counter" in "Household" at "@home" estimate "15m" created 18 days ago
    And task "Cancel unused gym membership" in "Admin" at "@home" estimate "15m" note "Check cancellation period first" created 16 days ago
    And task "Mow the lawn" in "Household" at "@garden" estimate "45m" priority B created 14 days ago
    And task "Research better phone plan" in "Admin" at "@home" estimate "45m" created 12 days ago
    And task "Fix squeaky hallway door" in "Household" at "@home" estimate "30m" created 10 days ago
    And task "File tax documents" in "Admin" at "@home" estimate "1h" priority A note "Deadline is next month" created 9 days ago
    And task "Water the houseplants" in "Household" at "@home" estimate "15m" created 7 days ago
    And task "Meal prep for the week" in "Household" at "@home" estimate "2h" created 6 days ago
    And task "Book car inspection" in "Admin" at "@phone" priority B created 5 days ago
    And task "Reply to school email about field trip" in "Kids" at "@home" estimate "15m" created 4 days ago
    And task "Buy birthday present for Emma" in "Family" at "@errands" estimate "30m" note "She mentioned she likes art supplies" created 3 days ago
    And task "Clean out the fridge" in "Household" at "@home" estimate "30m" created 2 days ago
    And task "Take out the recycling" in "Household" at "@home" estimate "15m" created 1 days ago
    And task "Plan something for the long weekend" in "Family" at "@home" created 1 days ago
    And completed task "Paid electricity bill" in "Admin" created 8 days ago
    And completed task "Picked up prescription" in "Errands" created 6 days ago
    And completed task "Fixed leaky bathroom faucet" in "Household" created 4 days ago
    And completed task "Helped Leo with science project" in "Kids" created 3 days ago
    And completed task "Brought bottles to recycling center" in "Household" created 1 days ago
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
