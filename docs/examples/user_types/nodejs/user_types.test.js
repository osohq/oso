const { oso } = require("./02-userClasses");

async function loadFile(example) {
  await oso.loadFile(example);
  return oso;
}

test("parses", () => {
  expect(loadFile("../user_policy.polar")).resolves.not.toThrow();
});
