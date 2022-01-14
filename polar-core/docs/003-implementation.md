# Goals

Encapsulated by the trait:

```rust
trait Goal {
    type Results: Iterator<Item = State>;
    fn run(self, state: State) -> Self::Results;
}
```

You run a goal, and as a result end up with an iterator
of states representing the possible results of executing the goal.

## Goals vs AST

In Polar, we can basically capture the entire AST as a goal:

```polar
f(x) if x = 1 or x = 2 + 3;

?= f(5)
```

evalutes to a goal:

```rust
 Call { f(5)} -> x=5 and x = 1 or x = 2+3

 And { Unify { x, 5}, Or { Unify { x, 1}, Unify { x, Add { 2, 3 }}}}
 
```