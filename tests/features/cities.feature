Feature: City Growth

  Scenario: A capital on fertile land grows in two turns
    Given the English settle on fertile land
    When the English found a city named London
    And exactly 1 turn ends
    Then the city of London is still size 1
    When exactly 1 turn ends
    Then the city of London grows to size 2
    And the English report that the city of London grows

  Scenario: A capital on plentiful land never grows
    Given the English settle on plentiful land
    When the English found a city named London
    When exactly 2 turns end
    Then the city of London is still size 1

  Scenario: A capital on barren land starves
    Given the English settle on land that offers no food
    When the English found a city named London
    When exactly 1 turn ends
    Then the city of London is still size 1
    And the English report that the city of London is starving

  Scenario: A capital starts producing a militia
    Given the English settle on fertile land
    When the English found a city named London
    And the English set London to produce a militia
    Then the city of London begins producing a militia is reported
    And the city of London is producing a militia

  Scenario: A militia is built in five turns on forest land
    Given the English settle on forest land
    When the English found a city named London
    And the English set London to produce a militia
    When exactly 5 turns end
    Then the city of London producing a militia is reported
    And the English now have a militia unit