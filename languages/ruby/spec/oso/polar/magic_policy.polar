allow(wiz: Wizard, "groom", fam: Familiar) if
  fam.wizard.name = wiz.name; # fam.wizard = wiz; # ???

allow(fam: Familiar, "groom", wiz: Wizard) if
  allow(wiz, "groom", fam) and
  fam.kind = "rat";

allow(fam: Familiar, "groom", _: Familiar) if
  fam.kind = "rat";

allow(wiz: Wizard, "ride", fam: Familiar) if
  allow(wiz, "groom", fam) and
  fam.kind = "horse";

allow(wiz: Wizard, "cast", spell: Spell) if
  spell.school in wiz.books and
  spell.level in wiz.spell_levels;

allow(fam: Familiar, "cast", spell: Spell) if
  allow(fam.wizard, "cast", spell);

allow(wiz: Wizard, "inscribe", spell: Spell) if
  allow(wiz, "cast", spell) and
  "inscription" in wiz.books;
