# Requirements: PAT-1, PAT-3, PAT-6, OFF-1, OFF-3
Feature: Register and re-triage patients

  Background:
    Given an active incident "Stadtfest Musterstadt" with the on-site care unit "UHS 1"
    And the medic "anna" works on a tablet assigned to "UHS 1"

  Scenario: Register a patient with minimal data
    When anna scans the triage tag "DE-0042" and selects category "SK II"
    Then the patient "DE-0042" appears on the UHS board of "UHS 1" with category "SK II"
    And the registration needed no other data

  Scenario: Register a patient while offline
    Given anna's tablet has no network connection
    When anna registers the patient "DE-0043" with category "SK III"
    Then the patient appears on anna's UHS board immediately
    When the tablet is connected again
    Then the patient "DE-0043" appears on all devices entitled to clinical data

  Scenario: Re-triage keeps the history
    Given the patient "DE-0042" has category "SK II"
    When anna changes the category to "SK I"
    Then the patient "DE-0042" has category "SK I"
    And the history of "DE-0042" shows "SK II" followed by "SK I", each with time and author
