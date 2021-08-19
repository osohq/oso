allow(wiz: Wizard, "groom", fam: Familiar) if
  fam in wiz.familiars;

allow(fam: Familiar, "groom", wiz: Wizard) if
  wiz = fam.wizard and
  fam.kind = "rat";

allow(fam: Familiar, "groom", other: Familiar) if
  fam.kind = "rat" and
  # data filtering doesn't support != yet :(
  other.kind in ["rat", "horse", "dwarf"];

allow(wiz: Wizard, "ride", fam: Familiar) if
  wiz = fam.wizard and
  fam.kind = "horse";

allow(wiz: Wizard, "cast", spell: Spell) if
  spell.school in wiz.books and
  spell.level in wiz.spell_levels;

allow(fam: Familiar, "cast", spell: Spell) if
  allow(fam.wizard, "cast", spell);

allow(wiz: Wizard, "inscribe", spell: Spell) if
  allow(wiz, "cast", spell) and
  "inscription" in wiz.books;
