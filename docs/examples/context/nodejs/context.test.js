const { oso } = require('./02-context');

test('01-context', async () => {
  await oso.loadFile('../01-context.polar');
  process.env['ENV'] = 'production';
  expect(await oso.isAllowed('steve', 'test', 'policy')).toBe(false);
  process.env['ENV'] = 'development';
  expect(await oso.isAllowed('steve', 'test', 'policy')).toBe(true);
});
