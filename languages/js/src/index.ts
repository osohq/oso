import { Oso } from './Oso';

async function main() {
  try {
    const oso = new Oso();
    await oso.loadFile('../test.polar');
    console.log(oso.query('isTrue()'));
  } catch (e) {
    console.log(e);
  }
}

main();
