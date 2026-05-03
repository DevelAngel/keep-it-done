@smoke @wasm
Feature: WASM Hydration Smoke Test

  Client-side interactions require WASM hydration.
  If clicking the next-view arrow has no effect, WASM failed to load.

  Scenario: View switch proves WASM hydration works
    When I open the app
    Then I see the page title is Upcoming
    When I click the next view arrow
    Then I see the page title is Quick Wins
