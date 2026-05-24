Feature: Session expired shows friendly message

  When the auth proxy (Tinyauth) rejects a server-function call
  with HTTP 401, the app shows a human-readable hint instead of
  a raw deserialization error.

  Scenario: Expired session shows re-login hint
    Given the following tasks
      | summary    | category | status | days ago |
      | Some task  | Inbox    | open   | 1        |
    When I open the app
    And the auth proxy rejects server requests
    And I click the next view arrow
    Then I see the session expired message
    And I save a screenshot as "session-expired"
