---
isAdmin: is_admin
authorize: authorize
authorizeField: authorize_field
authorizedFields: authorized_fields
lastCheckInLocation: |-
  ```python
  def get_last_check_in_location(profile, current_user):
      oso.authorize_field(current_user, "read", profile, "email")
      return profile.email
  ```
serializeProfile: |-
  ```python
  # Serialize only the fields of profile that the current user is allowed to read
  def serialize_profile(profile, current_user):
      fields = oso.authorized_fields(current_user, "read", profile)
      return {field: profile[field] for field in fields}
  ```
filterUpdateParams: |-
  ```python
  # Filter update_params by the fields on profile that the user can update
  def filter_update_params(profile, raw_update_params, current_user):
      fields = oso.authorized_fields(current_user, "update", profile)
      return {field: raw_update_params[field] for field in fields}
  ```
fieldsFriendsOnlyBefore: '["last_check_in_location", "favorite_animal"]'
fieldsFriendsOnlyAfter: Profile.FRIENDS_ONLY_FIELDS
fieldsAdminOnlyBefore: '["email", "last_login"]'
fieldsAdminOnlyAfter: Profile.ADMIN_ONLY_FIELDS
fieldDefinitions: |-
  Doing so would require you to add the `FRIENDS_ONLY_FIELDS` and
  `ADMIN_ONLY_FIELDS` constants to your `Profile` class:

  ```python
  class Profile:
      ADMIN_ONLY_FIELDS = ["email", "last_login"]
      FRIENDS_ONLY_FIELDS = ["last_check_in_location", "favorite_animal"]
  ```
---
