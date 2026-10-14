# Ref cases

The ref family pins design §3.1: how a ref becomes the file, folder or doc it names. The topics follow the resolution order, then the duplicate and reserved-name rules, then the round trip of printed labels.

## ABS

- `REF-ABS-01` An absolute path that names no entry is not found: stdout is empty, stderr names the path as typed, exit 1.
  Scenario: show /memex-no-such-folder/nope.rs in a repo. design §3.1 step 1 and §3.6 exit codes.

## REL

- `REF-REL-01` A relative path that only the root holds is found from a subfolder, and its section is labelled by its root-relative path, exit 0.
  Scenario: show src/cli.rs with cwd repo/src/deep, which holds no src/cli.rs of its own. design §3.1 step 2.
- `REF-REL-02` A `./` prefix and repeated slashes name the same entry, so ./src//cli.rs shows src/cli.rs, exit 0.
  Scenario: show ./src//cli.rs in the root. design §3.1 step 2.
- `REF-REL-03` A relative path that climbs above the root and names no entry is not found, exit 1.
  Scenario: show ../nope.rs in the root, where no folder beside the root holds it. design §3.1 steps 2 and 6.
- `REF-REL-04` A relative path that climbs out of the root and back in names the entry inside it, labelled by its root-relative path, exit 0.
  Scenario: show ../../repo/src/cli.rs with cwd repo/src. design §3.1 step 2: the path names the entry it reaches.

## AMBIG

- `REF-AMBIG-01` From the root, a relative path that the cwd and the root both hold names one entry, because they are the same folder, exit 0.
  Scenario: show README.md in the root. design §3.1 step 2.
- `REF-AMBIG-02` A bare id that two CASES.md files declare is ambiguous: stdout is empty, stderr names the ref and lists both CASES.md paths, exit 1.
  Scenario: show DUP-001, with CASES.md and sub/CASES.md both declaring it. design §4.2 and §3.1 steps 4 and 6.
- `REF-AMBIG-03` A bare id that names docs in two collections is ambiguous, and its candidates are their labels, sorted, exit 1.
  Scenario: show NAME-001, with the case NAME-001 and the doc doom:NAME-001 of a markdown manifest. design §3.1 steps 4 and 6.

## COLL

- `REF-COLL-01` A `collection:id` label names the doc of a manifest collection and prints it under that label, exit 0.
  Scenario: show doom:NOTE-1, where notes/NOTE-1.md is a markdown doc of the doom manifest. design §3.1 step 3 and §4.2.
- `REF-COLL-02` A collection prefix is never dropped: doom:NAME-001 is not found, though the id NAME-001 exists in cases, exit 1.
  Scenario: show doom:NAME-001 in the same repo as REF-COLL-01, where only the case NAME-001 holds that id. design §3.1 step 3, exact match, then step 6.

## BARE

- `REF-BARE-01` A bare log id, the file name of an entry without `.md`, names that entry under its `stream:id` label, exit 0.
  Scenario: show 20261013-120000-show-walks-hub-readmes, with that entry in the impl stream. design §3.1 step 4.
- `REF-BARE-02` A bare id that differs from a case id only in case names that case when no id matches exactly, exit 0.
  Scenario: show name-001 with the case NAME-001 declared. design §3.1 step 4, case-insensitive.
- `REF-BARE-03` An exact bare id match is taken before a case-insensitive one, so note-a names notes:note-a even though notes:Note-A exists, exit 0.
  Scenario: a markdown manifest whose id is a frontmatter field, with two docs whose ids differ only in case. design §3.1 step 4.

## BASE

- `REF-BASE-01` A basename that exactly one walked file has names that file, labelled by its root-relative path, exit 0.
  Scenario: show cli.rs in the root, where src/cli.rs is the only cli.rs. design §3.1 step 5.
- `REF-BASE-02` A basename that several walked files have is ambiguous, and its candidates are their root-relative paths, sorted, exit 1.
  Scenario: show cli.rs where src/cli.rs and tools/cli.rs exist. design §3.1 steps 5 and 6.
- `REF-BASE-03` A folder is not a basename match, so show deep, a folder name with no walked file of that name, is not found, exit 1.
  Scenario: show deep where src/deep is a folder and no walked file is named deep. design §3.1 step 5, which matches walked files only.

## CAND

- `REF-CAND-01` A candidate that an ambiguous basename prints round-trips: fed back to show from the root, it names its own file, exit 0.
  Scenario: show cli.rs (ambiguous), then show tools/cli.rs as it was printed. design §3.1 step 6 and the round-trip rule of §3.1.

## WT

- `REF-WT-01` A path outside the root that a linked worktree holds resolves against that worktree's own root: its section is labelled by its absolute path, and its chain lists that worktree's READMEs up to its root, exit 0. The root itself does not hold the file, so only the worktree can name it, and show src/cli.rs from the root is not found, exit 1.
  Scenario: git worktree add ../wt, then git rm src/cli.rs in repo; show ../wt/src/cli.rs from repo, then show src/cli.rs from repo. design §3.1 other worktrees.
- `REF-WT-02` A folder outside the root resolves against its own root too, so show ../wt/src prints that worktree's folder section, labelled by its absolute path, exit 0.
  Scenario: the same linked worktree, with the src folder. design §3.1 other worktrees and §3.3 folder.
- `REF-WT-03` A path outside every root that names no entry is not found, exit 1, though a linked worktree exists.
  Scenario: show ../wt/nope.rs from repo. design §3.1 other worktrees and §3.6.

## RESERVED

- `REF-RESERVED-01` readmes:src/README.md names the README by its literal path in the readmes collection, so it prints the section that show src/README.md prints, exit 0.
  Scenario: show readmes:src/README.md, then show src/README.md. design §3.1 reserved names; design is silent on the spelling, so the literal path is used.
- `REF-RESERVED-02` headers:src/cli.rs names the header doc by its literal path, so it prints the section that show src/cli.rs prints, exit 0.
  Scenario: show headers:src/cli.rs, then show src/cli.rs. design §3.1 reserved names; the literal path, as in REF-RESERVED-01.
- `REF-RESERVED-03` logs:ID names the log entry with that id in whichever stream holds it, labelled stream:ID, exit 0.
  Scenario: show logs:20261013-120000-show-walks-hub-readmes, with the entry in the impl stream. design §3.1 reserved names: logs means every stream.
- `REF-RESERVED-04` logs:ID that two streams both hold is ambiguous, and its candidates are both labels, sorted, exit 1.
  Scenario: the same stem in the impl and review streams. design §3.1 reserved names and step 6.

## TRIP

- `REF-TRIP-01` The label that show prints for a log entry, impl:ID, fed back to show, names the same entry as the bare id did, exit 0.
  Scenario: show 20261013-120000-show-walks-hub-readmes, then show impl:20261013-120000-show-walks-hub-readmes. design §3.1 labels round-trip.
