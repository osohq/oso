actor User {}
resource Repository {}

has_permission(_user: User, "read", repository: Repository) if
	repository.is_public = true;
