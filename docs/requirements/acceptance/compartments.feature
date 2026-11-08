# Requirements: SEC-8, SEC-9, PAT-6, CONN-3
Feature: Command sees patient counts but never patient data

  Background:
    Given an active incident with the on-site care unit "UHS 1"
    And the patients "DE-0042" (SK I), "DE-0043" (SK II) and "DE-0044" (SK II) are in "UHS 1"
    And the incident commander "ben" works on a tablet without access to clinical data

  Scenario: Counts reach the command tablet
    When ben's tablet syncs the incident
    Then ben sees for "UHS 1": SK I 1, SK II 2, SK III 0

  Scenario: Clinical data never reaches the command tablet
    When ben's tablet syncs the incident
    Then the local storage of ben's tablet contains no event of the compartment "med.clinical"
    And it contains no triage tag number and no patient name
