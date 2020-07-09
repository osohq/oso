# father(x, y) ⇒ y is the father of x.
father("Artemis", "Zeus");
?= father("Artemis", "Zeus");
father("Apollo", "Zeus");
father("Asclepius", "Apollo");
father("Aeacus", "Apollo");

# mother(x, y) ⇒ y is the mother of x.
mother("Apollo", "Leto");
mother("Artemis", "Leto");

# parent(x, y) ⇒ y is a parent of x.
parent(x, y) if father(x, y);
parent(x, y) if mother(x, y);

# grandfather(x, y) ⇒ y is a grandfather of x.
grandfather(x, y) if parent(x, p), father(p, y);

# ancestor(x, y) ⇒ y is an ancestor of x.
ancestor(x, y) if parent(x, y);
ancestor(x, y) if parent(x, p), ancestor(p, y);
