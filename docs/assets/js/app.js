class User {
    constructor(username) {
        this.username = username;
    }
}
class Expense {
    constructor(owner, category, amount) {
        this.owner = owner;
        this.category = category;
        this.amount = amount;
    }
}

const tryItButton = document.getElementById("tryitbutton");
if (tryItButton) {
    tryItButton.addEventListener("click", async (e) => {
        // console.log(e)
        // var oso = new window.oso.Oso();
        // oso.registerClass(User);
        // oso.registerClass(Expense);
        // var input = document.getElementById("tryitin").innerText;
        // var jsInput = document.getElementById("tryitinjs").innerText;
        // var actor;
        // var action;
        // var resource;
        // var out = eval(jsInput);
        // oso.loadStr(input);
        // var res = await oso.isAllowed(actor, action, resource);
        // document.getElementById("tryitout").innerText = res ? "True" : "False"
    })
}