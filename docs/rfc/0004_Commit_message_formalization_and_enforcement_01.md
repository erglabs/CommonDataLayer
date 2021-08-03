# Front Matter

```
Title           : Commit message formalization and enforcement
Author(s)       : Łukasz Biel
Team            : CommonDataLayer
Reviewer        : CommonDataLayer
Created         : 2021-03-17
Last updated    : 2021-03-17
Version         : 1.0.0
```

# Commit message formalization:

## Goal:
To have some standard. It can't be too strict because we are a small team, and this adds extra overhead.

## What we do now:
We are using vaguely defined `angular commit message spec.` This means our commits have titles with some markings, and that's it.

Tags that are used so far:
`test`
`ci`
`chore`
`refactor`
`fix`
`feature`
`docs`
`rfc`

Some of these overlap, we foregone using the scope of the commit.

## What we should be doing:
Document the process. Create a set of `tags` that are in use.
Some tags are duplicates. Thus I propose to use only:

* `chore` - dependency updates, ci updates, refactorings, other changes that don't affect `CDL` functionality
* `test` - adding or removing tests; it does not include changes to deployment or CI. These go under `chore.`
* `fix` - fixing a bug in **code**
* `feat` - adding new feature to **CDL**
* `docs` - adding documentation (readmes, mdbook, plantuml etc.)

On merge, we will squash commits and use one of 5 tags. We will not be using the scope as it's basically useless - most of our changes affect everything.
The summary should describe which components were changed.

In the meantime, we started enforcing names on PR's.
I propose that we drop tags in PR titles altogether and use labels.

As for the long commit message:

for dependabot - we can leave them as is.
For our work - we should make sure that commits titled wip aren't included; thus, it may be a good idea to have some standard there.

I propose we use the format as follows:

```
parent-tag: summary

* tag: summary
* tag: summary
```

Parent tag is the main goal of the work, and the list contains all things we did during implementation. E.g.:

```
feature: add lazers to CDL

* chore: add deployment target on the moon
* docs: document usage of lazers (and how to keep them away from cats)
```

# Commit message enforcement
This is a tough topic. We cannot enforce anything via default GitHub means. There are two options:

* We enforce a standard on ourselves and take care of it.
* We build a bot and perform merges only via its API.

The first proposal is what we do now. We sometimes miss a commit, but with a concrete set of guidelines (see above), we should not have conflicts.
The second proposal requires extra work. We could use a bot at some point for other things than merging as well.
We can create a task with a clear description of what it would do and backlog it.
