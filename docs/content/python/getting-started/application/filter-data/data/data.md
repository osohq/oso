---
queryObjectExampleLink: https://docs.sqlalchemy.org/en/14/orm/query.html#sqlalchemy.orm.Query
dataFilteringPath: examples/add-to-your-application/python/app/data_filtering.py

repoListQuerySnippet: |
    ```python
    @app.route("/repos")
    def repo_list():
        query = oso.authorized_query(
            User.get_current_user(),
            "read",
            Repository)

        # Use the ORM's Query API to alter the query before it is
        # executed by the database with .all().
        repositories = query.order_by(Repository.name).all()

        return serialize(repositories)
    ```

---
