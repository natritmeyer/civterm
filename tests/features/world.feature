Feature: The Starting World

  Scenario: A settled game sits on a standard map
    Given the English settle on fertile land
    Then the map is 80 tiles wide and 50 tiles tall
    And it is turn 1
    And the year is 4000 BC

  Scenario: The settlers start with a full treasury
    Given the English settle on fertile land
    Then the treasury holds 50 gold

  Scenario: The ground under and around a settler is revealed
    Given the English settle on fertile land
    Then the English starting tile is explored
    And the land around the English starting tile is explored