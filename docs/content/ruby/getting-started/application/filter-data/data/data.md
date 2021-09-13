---
queryObjectExampleLink: "http://sequel.jeremyevans.net/rdoc/files/README_rdoc.html#label-Filtering+Records"
dataFilteringPath: examples/add-to-your-application/ruby/data_filtering.rb

repoListQuerySnippet: |
    ```ruby
    get "/repos" do
      query = oso.authorized_query(
        get_current_user(),
        "read",
        Repository)

      # Use the ORM's Query API to alter the query before it is
      # executed by the database with .all.
      repositories = query.order_by(:name).all

      serialize(repositories)
    end
    ```

---
