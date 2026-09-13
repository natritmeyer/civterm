Feature: Standing Orders

  Scenario: A settler fortifies
    Given the English settle on fertile land
    When the English fortify the settler
    Then the English settler is fortified

  Scenario: A settler stands sentry
    Given the English settle on fertile land
    When the English put the settler on sentry
    Then the English settler is on sentry

  Scenario: A road is built in a single turn of work
    Given the English settle on rocky hills
    When the English begin building a road
    Then the English settler beginning work on a road is reported
    And the tile the English settler stands on is not roaded
    When exactly 1 turn ends
    Then the tile the English settler stands on is roaded
    And the English settler finishing a road is reported

  Scenario: A mine is hewn in two turns of work
    Given the English settle on rocky hills
    When the English begin building a mine
    Then the English settler beginning work on a mine is reported
    And the tile the English settler stands on is not mined
    When exactly 2 turns end
    Then the tile the English settler stands on is mined
    And the English settler finishing a mine is reported

  Scenario: An irrigation is dug in a single turn of work
    Given the English settle on fertile grassland
    When the English begin building an irrigation
    Then the English settler beginning work on an irrigation is reported
    And the tile the English settler stands on is not irrigated
    When exactly 1 turn ends
    Then the tile the English settler stands on is irrigated
    And the English settler finishing an irrigation is reported

  Scenario: An order is cancelled
    Given the English settle on fertile land
    When the English begin building a road
    And the English cancel the settler's order
    Then the English settler's order is cancelled