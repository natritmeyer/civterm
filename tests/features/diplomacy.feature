Feature: Diplomacy

  Scenario: War is declared and a second declaration is refused
    Given a new game with the English, the Zulu and the Roman
    When the English declare war on the Zulu
    Then the English declaring war on the Zulu is reported
    When the English declare war on the Zulu
    Then the English report that they are already at war with the Zulu

  Scenario: Peace is made and a second treaty is refused
    Given a new game with the English, the Zulu and the Roman
    When the English declare war on the Zulu
    And the English make peace with the Zulu
    Then the English making peace with the Zulu is reported
    When the English make peace with the Zulu
    Then the English report that they are already at peace with the Zulu

  Scenario: No war or peace with yourself
    Given a new game with the English, the Zulu and the Roman
    When the English declare war on the English
    Then the English declaring war on themselves is refused
    When the English make peace with the English
    Then the English making peace with themselves is refused