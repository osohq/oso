---
isAdmin: isAdmin
authorize: authorize
authorizeField: authorizeField
authorizedFields: authorizedFields
getEmail: |-
  ```java
  public String getEmail(Profile profile, User currentUser) throws OsoError {
      oso.authorizeField(currentUser, "read", profile, "email");
      return profile.email;
  }
  ```
serializeProfile: |-
  ```java
  // Serialize only the fields of profile that the current user is allowed to read
  public Map<String, Object> serializeProfile(Map<String, Object> profile, User currentUser) {
      Set<String> fields = new HashSet(oso.authorizedFields(currentUser, "read", profile));
      Map<String, Object> result = new HashMap();
      for (String field : fields) {
        result.put(field, profile.get(field));
      }
      return result;
  }
  ```
filterUpdateParams: |-
  ```java
  // Filter rawUpdateParams by the fields on profile that the user can update
  public Map<String, Object> filterUpdateParams(Profile profile, Map<String, Object> rawUpdateParams, User currentUser) {
      Set<String> fields = new HashSet(oso.authorizedFields(currentUser, "update", profile));
      Map<String, Object> result = new HashMap();
      for (String field : fields) {
        result.put(field, rawUpdateParams.get(field));
      }
      return result;
  }
  ```
fieldsFriendsOnlyBefore: '["lastCheckInLocation", "favoriteAnimal"]'
fieldsFriendsOnlyAfter: FRIENDS_ONLY_FIELDS
fieldsAdminOnlyBefore: '["email", "lastLogin"]'
fieldsAdminOnlyAfter: ADMIN_ONLY_FIELDS
fieldDefinitions: |-
  Doing so would require you to define the `FRIENDS_ONLY_FIELDS` and
  `ADMIN_ONLY_FIELDS` constants, and register them with Oso:

  ```java
  class Profile {
    static String[] FRIENDS_ONLY_FIELDS = new String[]{"lastCheckInLocation", "favoriteAnimal"};
    static String[] ADMIN_ONLY_FIELDS = new String[]{"email", "lastLogin"};
  }

  oso.RegisterConstant(Profile.ADMIN_ONLY_FIELDS, "ADMIN_ONLY_FIELDS")
  oso.RegisterConstant(Profile.FRIENDS_ONLY_FIELDS, "FRIENDS_ONLY_FIELDS")
  ```
---
