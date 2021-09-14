---
queryObjectExampleLink: "https://sequelize.org/master/manual/model-querying-basics.html#applying-where-clauses"
dataFilteringPath: examples/add-to-your-application/node/dataFiltering.js

repoListQuerySnippet: |
    ```javascript
    app.get('/repos', async (req, res) => {
      const where = await res.locals.oso.authorizedQuery(
        getCurrentUser(),
        'read',
        Repository
      );

      const repositories = await Repository.findAll({
        where,
        order: ['name']
      });

      res.end(serialize(repositories));
    });
    ```

---
