---
githubApp: "[Node.js sample app](https://github.com/osohq/oso-nodejs-quickstart)"
githubURL: "https://github.com/osohq/oso-nodejs-quickstart.git"
installation: |
    Install the project dependencies with NPM (or Yarn), then run the server:
    ```bash
    $ npm install
    installing requirements
    
    $ npm start
    server running on port 5050
    ```
amount: amount
manager: manager
submitted_by: submittedBy
endswith: endsWith
endswithURL: >
   [the `String.prototype.endsWith` method](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/String/endsWith)
expensesPath1: examples/quickstart/polar/expenses-01-nodejs.polar
expensesPath2: examples/quickstart/polar/expenses-02-nodejs.polar
isAllowed: isAllowed
installation_new: |
    ```bash
    npm install --save oso
    ```
import: import
import_code: |
    ```js
    const { Oso } = require('oso')
    oso = Oso()
    oso.enableRoles()
    ```
load_policy: |
    ```js
    oso.loadFile("authorization.polar")
    ```
getroles: getRoles
classes: JavaScript classes
objects: JavaScript objects
methods: JavaScript methods
register_classes: |
    ```js
    oso.registerClass(Page)
    oso.registerClass(User)
    ```
app_code: |
    ```js
    const app = express()
    const port = 3000

    app.get('/page/:pageNum', async (req, res) => {
        const pageNum = req.params.pageNum
        const page = Page.getPage(pageNum)
        const user = User.getCurrentUser()
        if (await oso.isAllowed(user, "read", page)) {
            res.send(`<h1>A Page</h1><p>this is page ${page.pagenum}</p>`)
        } else {
            res.status(403)
            res.send('<h1>Sorry</h1><p>You are not allowed to see this page</p>')
        }
    })
    ```
---
