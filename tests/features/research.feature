Feature: Research

  Scenario: Construction completes in ten turns
    Given the English settle on plentiful land
    When the English found a city named London
    Then the English beginning research on Construction is reported
    And the English are researching Construction
    And the English research progress is 0
    When exactly 10 turns end
    Then the English have Construction
    And the English are researching nothing
    And the English discovering Construction is reported

  Scenario: The Wheel follows in another fifteen turns
    Given the English settle on plentiful land
    When the English found a city named London
    And the English begin researching the Wheel
    Then the English beginning research on Wheel is reported
    And the English research progress is 0
    When exactly 15 turns end
    Then the English have Wheel
    And the English are researching nothing
    And the English discovering Wheel is reported
