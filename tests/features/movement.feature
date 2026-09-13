Feature: Movement

  Scenario: A settler moves onto open land
    Given the English settle on open land to the north
    When the English move north
    Then the English settler is one tile north of where it started
    And the English moving north is reported
    And the English settler's tile is open land

  Scenario: Dense ground exhausts a settler
    Given the English settle on dense terrain to the north
    When the English move north
    Then the English moving north is reported
    And the English settler's tile is dense terrain

  Scenario: The sea stops a settler at the coast
    Given the English settle on sea to the west
    When the English move west
    Then the English settler stands where it started
    And the English report that the sea stops the settler

  Scenario: A spent settler cannot move again
    Given the English settle on open land to the north
    When the English move north
    And the English move north
    Then the English settler has no moves remaining
    And the English report that the settler is spent