Feature: The March of Years

  Background:
    Given the English settle on fertile land

  Scenario: The first turn is 4000 BC
    Then the year is 4000 BC
    And it is turn 1

  Scenario: Twenty turns pass
    When exactly 20 turns end
    Then the year is 3000 BC
    And it is turn 21

  Scenario: Sixty turns pass
    When exactly 60 turns end
    Then the year is 1000 BC
    And it is turn 61

  Scenario: Sixty-one turns pass
    When exactly 61 turns end
    Then the year is 975 BC
    And it is turn 62

  Scenario: A century passes
    When exactly 100 turns end
    Then the year is 1 AD
    And it is turn 101