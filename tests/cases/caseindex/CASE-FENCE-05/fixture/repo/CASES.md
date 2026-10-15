---
memex: cases
---
# Fence cases

```text
- `CLOSE-001` A tilde line in a backtick fence is code.
~~~
- `CLOSE-002` The tilde line above does not close the fence.
```

```text
- `CLOSE-003` The line below has other text after its marks.
```md
- `CLOSE-004` This item is code, since the line above did not close.
```
- `CLOSE-005` An item after the real closer is a case.

````text
- `CLOSE-006` A shorter line does not close a longer fence.
```
- `CLOSE-007` The shorter line above is code.
````
- `CLOSE-008` An item after the longer closer is a case.
