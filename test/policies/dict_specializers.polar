what_dict_is(_: {Genus: "canis"}, "canine");
what_dict_is(_: {Species: "canis lupus", Genus: "canis"}, "wolf");
what_dict_is(_: {Species: "canis familiaris", Genus: "canis"}, "dog");

what_class_is(_: Animal, "animal");
what_class_is(_: Animal{Genus: "canis"}, "canine");
what_class_is(_: Animal{Family: "canidae"}, "canid");
what_class_is(_: Animal{Species: "canis lupus", Genus: "canis"}, "wolf");
what_class_is(_: Animal{Species: "canis familiaris", Genus: "canis"}, "dog");
what_class_is(_: Animal{Species: s, Genus: "canis"}, s);

what_is(_: {Genus: "canis"}, "canine_dict");
what_is(_: {Species: "canis lupus", Genus: "canis"}, "wolf_dict");
what_is(_: {Species: "canis familiaris", Genus: "canis"}, "dog_dict");

what_is(_: Animal, "animal_class");
what_is(_: Animal{Genus: "canis"}, "canine_class");
what_is(_: Animal{Family: "canidae"}, "canid_class");
what_is(_: Animal{Species: "canis lupus", Genus: "canis"}, "wolf_class");
what_is(_: Animal{Species: "canis familiaris", Genus: "canis"}, "dog_class");
