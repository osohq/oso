---
isAdmin: admin?
authorize: authorize
authorizeField: authorize_field
authorizedFields: authorized_fields
getEmail: |-
  ```ruby {hl_lines=[2]}
  def get_email(profile, current_user)
    oso.authorize_field(current_user, "read", profile, "email")
    profile.email
  end
  ```
serializeProfile: |-
  ```ruby {hl_lines=[3]}
  # Serialize only the fields of profile that the current user is allowed to read
  def serialize_profile(profile, current_user)
    fields = oso.authorized_fields(current_user, "read", profile)
    profile.slice(*fields)
  end
  ```
filterUpdateParams: |-
  ```ruby {hl_lines=[3]}
  # Filter raw_update_params by the fields on profile that the user can update
  def filter_update_params(profile, raw_update_params, current_user)
    fields = oso.authorized_fields(current_user, "update", profile)
    raw_update_params.slice(*fields)
  end
  ```
fieldsFriendsOnlyBefore: '["last_check_in_location", "favorite_animal"]'
fieldsFriendsOnlyAfter: Profile.FRIENDS_ONLY_FIELDS
fieldsAdminOnlyBefore: '["email", "last_login"]'
fieldsAdminOnlyAfter: Profile.ADMIN_ONLY_FIELDS
fieldDefinitions: |-
  Doing so would require you to add the `FRIENDS_ONLY_FIELDS` and
  `ADMIN_ONLY_FIELDS` constants to your `Profile` class:

  ```ruby
  class Profile
    ADMIN_ONLY_FIELDS = ["email", "last_login"]
    FRIENDS_ONLY_FIELDS = ["last_check_in_location", "favorite_animal"]
  end
  ```
---
