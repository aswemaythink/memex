# search

The parts of `memex search` (design §3.2) are split from `../search.rs`, which runs the verb. It groups the docs, ranks each group by the words or lists it by the filters, and limits it. `listing.rs` builds a listing or roster group: its docs with logs newest first and then by label, cut to the limit, each roster line with its cited-by count, and each log with its age. `age_of` lives there too, because search.rs shows a log's age on its word hits as well.
