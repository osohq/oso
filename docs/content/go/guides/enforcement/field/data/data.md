---
isAdmin: IsAdmin
authorize: Authorize
authorizeField: AuthorizeField
authorizedFields: AuthorizedFields
getEmail: |-
  ```go
  func getEmail(profile Profile, currentUser User) string {
    err = oso.AuthorizeField(currentUser, "read", profile, "email")
    if err != nil {
      return nil
    }
    return profile.Email
  }
  ```
serializeProfile: |-
  ```go
  // Serialize only the fields of profile that the current user is allowed to read
  func serializeProfile(profile map[string]interface{}, currentUser User) map[string]interface{} {
    fields, _ := oso.AuthorizedFields(currentUser, "read", profile)
    result := make(map[string]interface{})
    for field, _ := range fields {
      result[field] = profile[field]
    }
    return result
  }
  ```
filterUpdateParams: |-
  ```go
  // Filter rawUpdateParams by the fields on profile that the user can update
  func filterUpdateParams(profile Profile, rawUpdateParams map[string]interface{}, currentUser User) map[string]interface{} {
    fields, _ := oso.AuthorizedFields(currentUser, "update", profile)
    result := make(map[string]interface{})
    for field, _ := range fields {
      result[field] = rawUpdateParams[field]
    }
    return result
  }
  ```
fieldsFriendsOnlyBefore: '["last_check_in_location", "favorite_animal"]'
fieldsFriendsOnlyAfter: FriendsOnlyFields
fieldsAdminOnlyBefore: '["email", "last_login"]'
fieldsAdminOnlyAfter: AdminOnlyFields
fieldDefinitions: |-
  Doing so would require you to define the `FriendsOnlyFields` and
  `AdminOnlyFields` constants, and register them with Oso:

  ```go
  const AdminOnlyFields = []string{"email", "last_login"}
  const FriendsOnlyFields = []string{"last_check_in_location", "favorite_animal"}

  oso.RegisterConstant(AdminOnlyFields, "AdminOnlyFields")
  oso.RegisterConstant(FriendsOnlyFields, "FriendsOnlyFields")
  ```
---
