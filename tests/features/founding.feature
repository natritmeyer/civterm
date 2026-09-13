Feature: Founding

  Scenario: A settler founds a capital on its starting tile
    Given the English settle on fertile land
    When the English found a city named London
    Then the city of London stands where the settler stood
    And the English have no settler left
    And the city of London is owned by the English

  Scenario: Founding starts research on Construction
    Given the English settle on fertile land
    When the English found a city named London
    Then the English beginning research on Construction is reported

  Scenario: Founding reveals the land around the fledgling city
    Given the English settle on fertile land
    When the English found a city named London
    Then the land two tiles to the east of the English starting tile is explored