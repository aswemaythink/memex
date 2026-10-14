# ref

The ref family covers design §3.1: how a ref becomes the file, folder or doc that it names, and how the labels `show` prints can be typed back in. Its prefix is `REF`. Each case folder holds a small intent layer like show's fixture, and the worktree cases add a linked worktree with `git worktree add`. Entries are grouped by the step of the resolution order they test, so ABS, REL, AMBIG, COLL, BARE and BASE follow steps 1 to 6, and RESERVED, WT, CAND and TRIP cover reserved names, other worktrees and the round trip of labels.
