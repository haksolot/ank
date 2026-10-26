"""The agent bench: seeded fixtures, an isolated agent runner, a transcript scorer.

Standard library only, outside the cargo workspace (ADR-bafbd28498cc).
"""

# Bumped whenever a change to the harness could move a score: the generator's
# output, the runner's command line, or what the scorer counts.
HARNESS_VERSION = "1"
