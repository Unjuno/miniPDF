# Bug Note: readme-missing-links

## Symptom
- README linked to documentation files that are not present in the repository, breaking navigation for changelog and testing references.

## Root Cause
- The README was updated with new link targets without adding the corresponding files.

## Fix
- Point README references to existing docs and remove links to missing files.

## Prevent Regression
- Confirm README links against the tracked docs list before publishing documentation updates.
