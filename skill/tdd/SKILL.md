---
name: ank-tdd
description: Drive an implementation test-first, red before green, against a claimed task's frozen criterion. Use when implementing a task in a repository with a .ank/ directory.
metadata:
  revision: "5c0133123d36"
---

# ank-tdd

A criterion frozen at claim is already a test list. This file turns it into
one, clause by clause, and writes each test before the code that answers it.

The ank skill is the contract and applies here in full. This file adds the
test-first policy only.

## The criterion is the specification

Freezing is what makes the criterion usable as one: it cannot move to fit
what you built. Read it whole with `ank show <id>`, then split it into
clauses -- one per statement something could check, not one per sentence --
and write the list down before the first edit. Every clause becomes at least
one test.

A clause you cannot phrase as a test is a clause you have not understood.
`ank log` what is ambiguous and say which reading you took; code that guesses
silently is the same guess with no record of it.

Read the constraints into the same list. `ank context <path>` over the paths
the work will touch returns rules about behaviour, and a rule about behaviour
is testable on the same terms as the criterion's own clauses.

## The loop

    red        write the test for one clause, run it, watch it fail
    green      write the least code that passes it, run the suite
    refactor   improve what you just wrote, suite stays green
    next       the next clause, until the list is empty

**A test that has never failed has proved nothing.** Watch the red, and read
it: the failure message is the one you will be handed by this test for the
rest of its life, and now is when it is cheap to make it name the behaviour.
A test green on its first run is either behaviour the tree already had --
fine, record that and move on -- or it is not testing what you think, which
is the case this step exists to catch.

**One clause at a time.** The suite is green between clauses, so when it
breaks you know which clause did it: the last one. Two in flight and you
bisect by hand.

`ank log "<discovery>"` as you learn it. A red that surprised you is the
finding, and it stops being recoverable the moment it goes green.

## Three ways to write tests that prove nothing

**Tautological tests.** The test compares the implementation against itself: it
mirrors the code's own arithmetic into the expected value, rebuilds the
structure under test to assert equality with it, or mocks the thing being
tested and asserts the mock was called. It is green by construction and stays
green through any change of behaviour whatever. Write the expected value as a
literal somebody could have written before the code existed.

**Horizontal slicing.** A layer is built across its full width before anything
above it runs -- every field of the parser, then every field of the store, then
the verb -- and nothing is exercised end to end until the last layer lands. The
lower two were sized by guessing what the upper one would want, and the guess
is discovered at the point where it is most expensive to correct. Cut
vertically instead: one clause, through every layer it needs, green, then the
next.

**Testing the implementation rather than the behaviour.** The test names a
private function, asserts on an intermediate value, or counts calls. It goes
red on a refactor that changed no behaviour, which teaches its readers to
delete tests, and it stays green on a rewrite that broke the behaviour it was
meant to hold. Assert what a caller can observe. A criterion phrased about a
binary is tested through that binary, at the outermost seam a caller actually
reaches; the same rule applies inward, at whatever seam the clause names.

## Where the method stops

No verb enforces any of this. `done` runs the declared verifiers and measures
the tree as it stands; the order the file was written in is not a fact it
holds, and nothing asks it to. There is no route to grade, deliberately: an
agent graded on its process learns to produce the process.

So the discipline is yours, and so is the only thing it buys. A test you
watched fail is the evidence you hold that green means something. A suite
that has only ever been green is a suite nobody has measured, and it will
report success in exactly the case you wrote it to catch.
