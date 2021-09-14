---
isAdmin: isAdmin
authorize: authorize
authorizeField: authorizeField
authorizedFields: authorizedFields
getEmail: |-
  ```javascript
  async function getEmail(profile, currentUser) {
      await oso.authorizeField(currentUser, "read", profile, "email");
      return profile.email;
  }
  ```
serializeProfile: |-
  ```javascript
  // Serialize only the fields of profile that the current user is allowed to read
  async function serializeProfile(profile, currentUser) {
      const fields = await oso.authorizedFields(currentUser, "read", profile);
      const result = {};
      for (const field of fields) {
        result[field] = profile[field];
      }
      return result;
  }
  ```
filterUpdateParams: |-
  ```javascript
  // Filter rawUpdateParams by the fields on profile that the user can update
  async function filterUpdateParams(profile, rawUpdateParams, currentUser) {
      const fields = await oso.authorizedFields(currentUser, "update", profile);
      const result = {};
      for (const field of fields) {
        result[field] = rawUpdateParams[field];
      }
      return result;
  }
  ```
fieldsFriendsOnlyBefore: '["lastCheckInLocation", "favoriteAnimal"]'
fieldsFriendsOnlyAfter: Profile.FRIENDS_ONLY_FIELDS
fieldsAdminOnlyBefore: '["email", "lastLogin"]'
fieldsAdminOnlyAfter: Profile.ADMIN_ONLY_FIELDS
fieldDefinitions: |-
  Doing so would require you to add the `FRIENDS_ONLY_FIELDS` and
  `ADMIN_ONLY_FIELDS` constants to your `Profile` class:

  ```javascript
  Profile.ADMIN_ONLY_FIELDS = ["email", "lastLogin"];
  Profile.FRIENDS_ONLY_FIELDS = ["lastCheckInLocation", "favoriteAnimal"];
  ```
---
