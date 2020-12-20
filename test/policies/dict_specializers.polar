what_dict_is(_: {genus: "canis"}, "canine");
what_dict_is(_: {species: "canis lupus", genus: "canis"}, "wolf");
what_dict_is(_: {species: "canis familiaris", genus: "canis"}, "dog");

what_class_is(_: Animal, "animal");
what_class_is(_: Animal{genus: "canis"}, "canine");
what_class_is(_: Animal{family: "canidae"}, "canid");
what_class_is(_: Animal{species: "canis lupus", genus: "canis"}, "wolf");
what_class_is(_: Animal{species: "canis familiaris", genus: "canis"}, "dog");
what_class_is(_: Animal{species: s, genus: "canis"}, s);

what_is(_: {genus: "canis"}, "canine_dict");
what_is(_: {species: "canis lupus", genus: "canis"}, "wolf_dict");
what_is(_: {species: "canis familiaris", genus: "canis"}, "dog_dict");

what_is(_: Animal, "animal_class");
what_is(_: Animal{genus: "canis"}, "canine_class");
what_is(_: Animal{family: "canidae"}, "canid_class");
what_is(_: Animal{species: "canis lupus", genus: "canis"}, "wolf_class");
what_is(_: Animal{species: "canis familiaris", genus: "canis"}, "dog_class");
