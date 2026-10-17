# cite

The cite family covers design §2.3: the citations a doc makes and the backlinks they give. It checks them as `memex show` prints them (§3.3 and §3.6). Its prefix is `CITE`, and each case folder is named by its id.

Each case folder carries a small intent layer as its fixture: a root README, a `src/` folder with a README and a header, and a CASES.md that declares the ids the case cites. A case that reads a log adds a `.memex/` store with one entry.

## Choices where design §2.3 is silent

- A prefix citation (`NAME-*`) is a backlink only. Each id it names lists the citing doc under cited by, marked `(via NAME-*)`, and the citing doc never gets a cites line for it: a family can hold hundreds of ids, and cites print without a limit (design §1, bounded output).
- A case section shows cited by only, and its JSON has an empty cites list, as design §3.3 Case and the caseindex family pin.
- A doc never cites itself, and a README never cites its own folder: both are self-citations and are dropped.
- A bare id resolves to every declared doc with that id, in label order. Declared means a CASES.md or a manifest holds the id, and the id has the case-id grammar.
- `collection:id` is explicit only when the collection is known. `readmes`, `headers` and `logs` always are (the names of design §3.1); any other name is known when some doc holds it, as a stream, a manifest or a frontmatter collection does. So a case that cites `cases:<ID>` declares a CASES.md, as CITE-COLL-02 does.
- A `collection:id` label resolves as `show` resolves a label (design §3.1). `readmes:` and `headers:` name a doc by its path, and `logs:` names the entry of every stream that holds the id. A label that names several docs cites each of them.
- A path token that names nothing is silent. Only explicit refs (markdown links and known `collection:id`) can be unresolved (design §2.3).
- A markdown link that climbs above the root is not a citation at all.
- A link or path token that names a collection doc's file cites the doc by its `collection:id` label, as CITE-MAN-01 pins, since that doc is the one a reader is sent to. A file that an intent doc holds, such as a README that a manifest also covers, keeps its path label, so its citers do not move.
- An entry of another root (an absolute path, or a path that climbs out of the cwd's root) prints `(none)` for cites and for cited by. Its citations name labels of that root, which do not round-trip from the cwd (design §3.1), so printing them needs absolute refs first.
